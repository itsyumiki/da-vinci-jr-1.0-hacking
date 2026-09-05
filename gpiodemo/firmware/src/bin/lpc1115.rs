#![cfg_attr(target_arch = "arm", no_std)]
#![cfg_attr(target_arch = "arm", no_main)]

#[cfg(not(target_arch = "arm"))]
fn main() {}

#[cfg(target_arch = "arm")]
mod board {
    use cortex_m_rt::entry;
    use da_vinci_firmware::{
        Node,
        lpc::{LPC_IDENTITY, LpcGpio, LpcUart},
    };
    use lpc11xx as pac;
    use panic_halt as _;

    const LOCAL_ROUTE: &[u8] = b"LPC";

    #[entry]
    fn main() -> ! {
        let peripherals = pac::Peripherals::take().unwrap();
        configure_clocks_and_uart(&peripherals.SYSCON, &peripherals.IOCON, &peripherals.UART);

        let mut gpio = LpcGpio::new(
            peripherals.IOCON,
            peripherals.GPIO0,
            peripherals.GPIO1,
            peripherals.GPIO2,
            peripherals.GPIO3,
        );
        let mut uart = LpcUart::new(peripherals.UART);
        let mut node = Node::new(LPC_IDENTITY, LOCAL_ROUTE, []);

        loop {
            let _ = node.poll(&mut uart, &mut gpio);
        }
    }

    fn configure_clocks_and_uart(syscon: &pac::SYSCON, iocon: &pac::IOCON, uart: &pac::UART) {
        syscon
            .sysahbclkctrl
            .modify(|_, w| w.gpio().set_bit().uart().set_bit().iocon().set_bit());
        // SAFETY: UARTCLKDIV accepts any non-zero divider; 1 selects the undivided main clock.
        syscon.uartclkdiv.write(|w| unsafe { w.div().bits(1) });

        iocon
            .iocon_pio1_6
            .modify(|_, w| w.func().rxd().mode().inactive_no_pull_do());
        iocon
            .iocon_pio1_7
            .modify(|_, w| w.func().txd().mode().inactive_no_pull_do());
        iocon.iocon_rxd_loc.write(|w| w.rxdloc().pio1_6());

        // 12 MHz IRC / 16 / 4 / (1 + 5/8) = 115384.6 baud (0.16% high).
        uart.lcr.write(|w| w.wls().eight().dlab().enable());
        uart.dll().write(|w| w.dllsb().bits(4));
        uart.dlm().write(|w| w.dlmsb().bits(0));
        uart.fdr.write(|w| w.divaddval().bits(5).mulval().bits(8));
        uart.lcr.write(|w| w.wls().eight().dlab().disable());
        uart.fcr()
            .write(|w| w.fifoen().enable().rxfifores().clear().txfifores().clear());
        uart.ter.write(|w| w.txen().set_bit());
    }
}
