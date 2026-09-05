#![cfg_attr(target_arch = "arm", no_std)]
#![cfg_attr(target_arch = "arm", no_main)]

#[cfg(not(target_arch = "arm"))]
fn main() {}

#[cfg(target_arch = "arm")]
mod board {
    use atsam4_hal::{
        clock::{ClockController, MainClock, SlowClock},
        gpio::{GpioExt, Ports},
        pac,
        serial::Uart1,
        udp::{UdpBus, usb_device},
        watchdog::{Watchdog, WatchdogDisable},
    };
    use cortex_m_rt::entry;
    use da_vinci_firmware::{
        Node,
        router::Route,
        sam::{SAM_IDENTITY, SamGpio, SamUartBytes},
        transport::{ByteError, FramedLink, NonBlockingBytes},
    };
    use panic_halt as _;
    use usb_device::{class_prelude::UsbBusAllocator, prelude::*};
    use usbd_serial::{SerialPort, USB_CLASS_CDC};

    const LOCAL_ROUTE: &[u8] = b"SAM";
    const LPC_DESTINATIONS: &[&[u8]] = &[b"LPC"];

    struct UsbBytes<'a, 'bus, B: usb_device::bus::UsbBus>(&'a mut SerialPort<'bus, B>);

    impl<B: usb_device::bus::UsbBus> NonBlockingBytes for UsbBytes<'_, '_, B> {
        fn try_read(&mut self, out: &mut [u8]) -> Result<usize, ByteError> {
            self.0.read(out).map_err(byte_error)
        }

        fn try_write(&mut self, bytes: &[u8]) -> Result<usize, ByteError> {
            self.0.write(bytes).map_err(byte_error)
        }
    }

    fn byte_error(error: usb_device::UsbError) -> ByteError {
        match error {
            usb_device::UsbError::WouldBlock => ByteError::WouldBlock,
            _ => ByteError::Down,
        }
    }

    #[entry]
    fn main() -> ! {
        let peripherals = pac::Peripherals::take().unwrap();
        let mut watchdog = Watchdog::new(peripherals.WDT);
        watchdog.disable();

        let mut clocks = ClockController::new(
            peripherals.PMC,
            &peripherals.SUPC,
            &peripherals.EFC,
            MainClock::Crystal12Mhz,
            SlowClock::RcOscillator32Khz,
        );
        let pio_a = clocks.peripheral_clocks.pio_a.into_enabled_clock();
        let pio_b = clocks.peripheral_clocks.pio_b.into_enabled_clock();
        let pio_c = clocks.peripheral_clocks.pio_c.into_enabled_clock();
        let pio_d = clocks.peripheral_clocks.pio_d.into_enabled_clock();
        let pio_e = clocks.peripheral_clocks.pio_e.into_enabled_clock();
        let udp_clock = clocks.peripheral_clocks.udp;
        let uart_1_clock = clocks.peripheral_clocks.uart_1.into_enabled_clock();

        let pins = Ports::new(
            (peripherals.PIOA, pio_a),
            (peripherals.PIOB, pio_b),
            (peripherals.PIOC, pio_c),
            (peripherals.PIOD, pio_d),
            (peripherals.PIOE, pio_e),
        )
        .split();
        let uart_rx = pins.pa5.into_peripheral_function_c(&peripherals.MATRIX);
        let uart_tx = pins.pa6.into_peripheral_function_c(&peripherals.MATRIX);
        let ddm = pins.pb10.into_system_function(&peripherals.MATRIX);
        let ddp = pins.pb11.into_system_function(&peripherals.MATRIX);

        let usb_bus = UsbBusAllocator::new(UdpBus::new(peripherals.UDP, udp_clock, ddm, ddp));
        let mut serial = SerialPort::new(&usb_bus);
        let mut usb = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x1d50, 0x614e))
            .device_class(USB_CLASS_CDC)
            .build();

        let uart = Uart1::new(
            peripherals.UART1,
            uart_1_clock,
            uart_rx,
            uart_tx,
            115_200,
            None,
        );
        let mut lpc_link = FramedLink::new(SamUartBytes::new(uart));
        let lpc_route = Route::new(b"LPC", LPC_DESTINATIONS, &mut lpc_link);
        let mut gpio = SamGpio;
        let mut node = Node::new(SAM_IDENTITY, LOCAL_ROUTE, [lpc_route]);

        loop {
            usb.poll(&mut [&mut serial]);
            let _ = node.poll(&mut UsbBytes(&mut serial), &mut gpio);
        }
    }
}
