#[cfg(any(test, all(target_arch = "arm", feature = "lpc1115")))]
use crate::gpio::map::{BankId, BankInfo, Capabilities, PinId, PinInfo, PinMap};

#[cfg(all(target_arch = "arm", feature = "lpc1115"))]
use core::ptr::{read_volatile, write_volatile};

#[cfg(all(target_arch = "arm", feature = "lpc1115"))]
use da_vinci_protocol::Level;
#[cfg(all(target_arch = "arm", feature = "lpc1115"))]
use lpc11xx as pac;

#[cfg(all(target_arch = "arm", feature = "lpc1115"))]
use crate::{
    gpio::{GpioHal, PinMode},
    transport::{ByteError, NonBlockingBytes},
};

#[cfg(any(test, all(target_arch = "arm", feature = "lpc1115")))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LpcBank {
    Pio0,
    Pio1,
    Pio2,
    Pio3,
}

#[cfg(any(test, all(target_arch = "arm", feature = "lpc1115")))]
impl LpcBank {
    pub(crate) const fn id(self) -> BankId {
        BankId::new(match self {
            Self::Pio0 => 0,
            Self::Pio1 => 1,
            Self::Pio2 => 2,
            Self::Pio3 => 3,
        })
    }

    #[cfg(all(target_arch = "arm", feature = "lpc1115"))]
    pub(crate) fn from_id(id: BankId) -> Option<Self> {
        match id.index() {
            0 => Some(Self::Pio0),
            1 => Some(Self::Pio1),
            2 => Some(Self::Pio2),
            3 => Some(Self::Pio3),
            _ => None,
        }
    }
}

#[cfg(any(test, all(target_arch = "arm", feature = "lpc1115")))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LpcPadKind {
    Standard,
    Analog,
    I2cOpenDrain,
}

#[cfg(any(test, all(target_arch = "arm", feature = "lpc1115")))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LpcPinHw {
    bank: LpcBank,
    bit: u8,
    iocon_offset: u8,
    gpio_function: u8,
    pad_kind: LpcPadKind,
}

#[cfg(any(test, all(target_arch = "arm", feature = "lpc1115")))]
impl LpcPinHw {
    const fn bank(self) -> LpcBank {
        self.bank
    }

    const fn bit(self) -> u8 {
        self.bit
    }

    #[cfg(all(target_arch = "arm", feature = "lpc1115"))]
    const fn iocon_offset(self) -> u8 {
        self.iocon_offset
    }

    #[cfg(all(target_arch = "arm", feature = "lpc1115"))]
    const fn gpio_function(self) -> u8 {
        self.gpio_function
    }

    #[cfg(all(target_arch = "arm", feature = "lpc1115"))]
    const fn pad_kind(self) -> LpcPadKind {
        self.pad_kind
    }
}

pub const LPC_IDENTITY: &[u8] = b"LPC1115 GPIO";

#[cfg(any(test, all(target_arch = "arm", feature = "lpc1115")))]
static BANKS: [BankInfo; 4] = [
    BankInfo::new("PIO0"),
    BankInfo::new("PIO1"),
    BankInfo::new("PIO2"),
    BankInfo::new("PIO3"),
];

#[cfg(any(test, all(target_arch = "arm", feature = "lpc1115")))]
macro_rules! lpc_pins {
    ($($token:literal => {
        package: $package:literal,
        bank: $bank:ident,
        bit: $bit:literal,
        iocon: $iocon:literal,
        function: $function:literal,
        kind: $kind:ident,
        caps: $caps:ident
    }),+ $(,)?) => {
        static PINS: &[PinInfo] = &[
            $(PinInfo::new(
                $token,
                Some($package),
                LpcBank::$bank.id(),
                $bit,
                Capabilities::$caps,
            )),+
        ];

        static LPC_HW: &[LpcPinHw] = &[
            $(LpcPinHw {
                bank: LpcBank::$bank,
                bit: $bit,
                iocon_offset: $iocon,
                gpio_function: $function,
                pad_kind: LpcPadKind::$kind,
            }),+
        ];
    };
}

#[cfg(any(test, all(target_arch = "arm", feature = "lpc1115")))]
lpc_pins! {
    "PIO0_0" => { package: 3, bank: Pio0, bit: 0, iocon: 0x0c, function: 0, kind: Standard, caps: NONE },
    "PIO0_1" => { package: 4, bank: Pio0, bit: 1, iocon: 0x10, function: 0, kind: Standard, caps: INPUT },
    "PIO0_2" => { package: 10, bank: Pio0, bit: 2, iocon: 0x1c, function: 0, kind: Standard, caps: INPUT },
    "PIO0_3" => { package: 14, bank: Pio0, bit: 3, iocon: 0x2c, function: 0, kind: Standard, caps: INPUT },
    "PIO0_4" => { package: 15, bank: Pio0, bit: 4, iocon: 0x30, function: 0, kind: I2cOpenDrain, caps: INPUT },
    "PIO0_5" => { package: 16, bank: Pio0, bit: 5, iocon: 0x34, function: 0, kind: I2cOpenDrain, caps: INPUT },
    "PIO0_6" => { package: 22, bank: Pio0, bit: 6, iocon: 0x4c, function: 0, kind: Standard, caps: INPUT },
    "PIO0_7" => { package: 23, bank: Pio0, bit: 7, iocon: 0x50, function: 0, kind: Standard, caps: INPUT },
    "PIO0_8" => { package: 27, bank: Pio0, bit: 8, iocon: 0x60, function: 0, kind: Standard, caps: INPUT },
    "PIO0_9" => { package: 28, bank: Pio0, bit: 9, iocon: 0x64, function: 0, kind: Standard, caps: INPUT },
    "PIO0_10" => { package: 29, bank: Pio0, bit: 10, iocon: 0x68, function: 0, kind: Standard, caps: NONE },
    "PIO0_11" => { package: 32, bank: Pio0, bit: 11, iocon: 0x74, function: 1, kind: Analog, caps: INPUT },
    "PIO1_0" => { package: 33, bank: Pio1, bit: 0, iocon: 0x78, function: 1, kind: Analog, caps: INPUT },
    "PIO1_1" => { package: 34, bank: Pio1, bit: 1, iocon: 0x7c, function: 1, kind: Analog, caps: INPUT },
    "PIO1_2" => { package: 35, bank: Pio1, bit: 2, iocon: 0x80, function: 1, kind: Analog, caps: INPUT },
    "PIO1_3" => { package: 39, bank: Pio1, bit: 3, iocon: 0x90, function: 0, kind: Analog, caps: NONE },
    "PIO1_4" => { package: 40, bank: Pio1, bit: 4, iocon: 0x94, function: 0, kind: Analog, caps: INPUT },
    "PIO1_5" => { package: 45, bank: Pio1, bit: 5, iocon: 0xa0, function: 0, kind: Standard, caps: INPUT },
    "PIO1_6" => { package: 46, bank: Pio1, bit: 6, iocon: 0xa4, function: 0, kind: Standard, caps: NONE },
    "PIO1_7" => { package: 47, bank: Pio1, bit: 7, iocon: 0xa8, function: 0, kind: Standard, caps: NONE },
    "PIO1_8" => { package: 9, bank: Pio1, bit: 8, iocon: 0x14, function: 0, kind: Standard, caps: INPUT },
    "PIO1_9" => { package: 17, bank: Pio1, bit: 9, iocon: 0x38, function: 0, kind: Standard, caps: INPUT },
    "PIO1_10" => { package: 30, bank: Pio1, bit: 10, iocon: 0x6c, function: 0, kind: Analog, caps: INPUT },
    "PIO1_11" => { package: 42, bank: Pio1, bit: 11, iocon: 0x98, function: 0, kind: Analog, caps: INPUT },
    "PIO2_0" => { package: 2, bank: Pio2, bit: 0, iocon: 0x08, function: 0, kind: Standard, caps: INPUT },
    "PIO2_1" => { package: 13, bank: Pio2, bit: 1, iocon: 0x28, function: 0, kind: Standard, caps: INPUT },
    "PIO2_2" => { package: 26, bank: Pio2, bit: 2, iocon: 0x5c, function: 0, kind: Standard, caps: INPUT },
    "PIO2_3" => { package: 38, bank: Pio2, bit: 3, iocon: 0x8c, function: 0, kind: Standard, caps: INPUT },
    "PIO2_4" => { package: 19, bank: Pio2, bit: 4, iocon: 0x40, function: 0, kind: Standard, caps: INPUT },
    "PIO2_5" => { package: 20, bank: Pio2, bit: 5, iocon: 0x44, function: 0, kind: Standard, caps: INPUT },
    "PIO2_6" => { package: 1, bank: Pio2, bit: 6, iocon: 0x00, function: 0, kind: Standard, caps: INPUT },
    "PIO2_7" => { package: 11, bank: Pio2, bit: 7, iocon: 0x20, function: 0, kind: Standard, caps: INPUT },
    "PIO2_8" => { package: 12, bank: Pio2, bit: 8, iocon: 0x24, function: 0, kind: Standard, caps: INPUT },
    "PIO2_9" => { package: 24, bank: Pio2, bit: 9, iocon: 0x54, function: 0, kind: Standard, caps: INPUT },
    "PIO2_10" => { package: 25, bank: Pio2, bit: 10, iocon: 0x58, function: 0, kind: Standard, caps: INPUT },
    "PIO2_11" => { package: 31, bank: Pio2, bit: 11, iocon: 0x70, function: 0, kind: Standard, caps: INPUT },
    "PIO3_0" => { package: 36, bank: Pio3, bit: 0, iocon: 0x84, function: 0, kind: Standard, caps: INPUT },
    "PIO3_1" => { package: 37, bank: Pio3, bit: 1, iocon: 0x88, function: 0, kind: Standard, caps: INPUT },
    "PIO3_2" => { package: 43, bank: Pio3, bit: 2, iocon: 0x9c, function: 0, kind: Standard, caps: INPUT },
    "PIO3_3" => { package: 48, bank: Pio3, bit: 3, iocon: 0xac, function: 0, kind: Standard, caps: INPUT },
    "PIO3_4" => { package: 18, bank: Pio3, bit: 4, iocon: 0x3c, function: 0, kind: Standard, caps: INPUT },
    "PIO3_5" => { package: 21, bank: Pio3, bit: 5, iocon: 0x48, function: 0, kind: Standard, caps: INPUT },
}

#[cfg(any(test, all(target_arch = "arm", feature = "lpc1115")))]
pub(crate) static LPC_PIN_MAP: PinMap = PinMap::new(&BANKS, PINS);

#[cfg(any(test, all(target_arch = "arm", feature = "lpc1115")))]
fn pin_hw(pin: PinId) -> &'static LpcPinHw {
    &LPC_HW[pin.index()]
}

#[cfg(all(target_arch = "arm", feature = "lpc1115"))]
pub struct LpcGpio {
    _iocon: pac::IOCON,
    gpio0: pac::GPIO0,
    gpio1: pac::GPIO1,
    gpio2: pac::GPIO2,
    gpio3: pac::GPIO3,
}

#[cfg(all(target_arch = "arm", feature = "lpc1115"))]
impl LpcGpio {
    pub const fn new(
        iocon: pac::IOCON,
        gpio0: pac::GPIO0,
        gpio1: pac::GPIO1,
        gpio2: pac::GPIO2,
        gpio3: pac::GPIO3,
    ) -> Self {
        Self {
            _iocon: iocon,
            gpio0,
            gpio1,
            gpio2,
            gpio3,
        }
    }

    fn registers(&self, bank: LpcBank) -> &pac::gpio0::RegisterBlock {
        match bank {
            LpcBank::Pio0 => &self.gpio0,
            LpcBank::Pio1 => &self.gpio1,
            LpcBank::Pio2 => &self.gpio2,
            LpcBank::Pio3 => &self.gpio3,
        }
    }

    fn configure_pad(&self, pin: PinId, pull_up: bool) {
        let hw = pin_hw(pin);
        let offset = hw.iocon_offset() as usize;
        // SAFETY: LpcGpio owns IOCON for its lifetime. Each PinId comes from LPC_PIN_MAP and has
        // matching hardware metadata. The PAC exposes IOCON as named registers rather than an
        // indexable array, so volatile pointer access is required for metadata-driven dispatch.
        unsafe {
            let register = (pac::IOCON::ptr() as *mut u8).add(offset).cast::<u32>();
            let current = read_volatile(register);
            let next = match hw.pad_kind() {
                LpcPadKind::I2cOpenDrain => (current & !(0x07 | (0x03 << 8))) | (1 << 8),
                LpcPadKind::Standard | LpcPadKind::Analog => {
                    let mode = if pull_up { 2 } else { 0 };
                    let mut bits = (current & !(0x07 | (0x03 << 3)))
                        | u32::from(hw.gpio_function())
                        | ((mode as u32) << 3);
                    if hw.pad_kind() == LpcPadKind::Analog {
                        bits |= 1 << 7;
                    }
                    bits
                }
            };
            write_volatile(register, next);
        }
    }
}

#[cfg(all(target_arch = "arm", feature = "lpc1115"))]
impl GpioHal for LpcGpio {
    fn pin_map(&self) -> &'static PinMap {
        &LPC_PIN_MAP
    }

    fn configure(&mut self, pin: PinId, mode: PinMode) {
        let hw = pin_hw(pin);
        let mask = 1u32 << hw.bit();
        match mode {
            PinMode::Input { pull_up } => {
                self.configure_pad(pin, pull_up);
                // DIR is a whole-bank bitmap in this PAC, so changing one pin requires a masked
                // raw update while preserving neighbouring direction bits.
                self.registers(hw.bank())
                    .dir
                    .modify(|r, w| unsafe { w.bits(r.bits() & !mask) });
            }
            PinMode::Output { initial } => {
                self.configure_pad(pin, false);
                self.write(pin, initial);
                self.registers(hw.bank())
                    .dir
                    .modify(|r, w| unsafe { w.bits(r.bits() | mask) });
            }
        }
    }

    fn write(&mut self, pin: PinId, level: Level) {
        let hw = pin_hw(pin);
        let mask = 1u32 << hw.bit();
        // DATA is likewise exposed as a whole-bank bitmap by this PAC.
        self.registers(hw.bank()).data.modify(|r, w| unsafe {
            w.bits(match level {
                Level::Low => r.bits() & !mask,
                Level::High => r.bits() | mask,
            })
        });
    }

    fn read_bank(&self, bank: BankId) -> u32 {
        self.registers(LpcBank::from_id(bank).expect("LPC pin map contains only GPIO0-3"))
            .data
            .read()
            .bits()
    }
}

#[cfg(all(target_arch = "arm", feature = "lpc1115"))]
pub struct LpcUart(pac::UART);

#[cfg(all(target_arch = "arm", feature = "lpc1115"))]
impl LpcUart {
    pub const fn new(uart: pac::UART) -> Self {
        Self(uart)
    }
}

#[cfg(all(target_arch = "arm", feature = "lpc1115"))]
impl NonBlockingBytes for LpcUart {
    fn try_read(&mut self, out: &mut [u8]) -> Result<usize, ByteError> {
        let mut read = 0;
        while read < out.len() && self.0.lsr.read().rdr().bit_is_set() {
            out[read] = self.0.rbr().read().rbr().bits();
            read += 1;
        }
        if read == 0 {
            Err(ByteError::WouldBlock)
        } else {
            Ok(read)
        }
    }

    fn try_write(&mut self, bytes: &[u8]) -> Result<usize, ByteError> {
        if bytes.is_empty() || !self.0.lsr.read().thre().bit_is_set() {
            return Err(ByteError::WouldBlock);
        }
        self.0.thr().write(|w| w.thr().bits(bytes[0]));
        Ok(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpio::Target;

    #[test]
    fn map_covers_the_48_pin_package_conservatively() {
        assert_eq!(LPC_PIN_MAP.banks().len(), 4);
        assert_eq!(LPC_PIN_MAP.pins().len(), 42);
        assert_eq!(LPC_HW.len(), LPC_PIN_MAP.pins().len());

        let mut package_pins = [false; 49];
        for (index, pin) in LPC_PIN_MAP.pins().iter().enumerate() {
            let token = pin.token.as_bytes();
            assert!(token.starts_with(b"PIO"));
            assert_eq!(token[3] - b'0', pin.bank.index() as u8);
            assert_eq!(pin.token[5..].parse::<u8>().unwrap(), pin.bit);

            let hw = pin_hw(PinId::new(index as u8));
            assert_eq!(hw.bank().id(), pin.bank);
            assert_eq!(hw.bit(), pin.bit);

            let package_pin = pin.package_pin.unwrap() as usize;
            assert!((1..=48).contains(&package_pin));
            assert!(!package_pins[package_pin]);
            package_pins[package_pin] = true;
        }

        for token in [
            b"PIO0_0".as_slice(),
            b"PIO0_10",
            b"PIO1_3",
            b"PIO1_6",
            b"PIO1_7",
        ] {
            let Target::Pin(pin) = LPC_PIN_MAP.resolve(token).unwrap() else {
                panic!("reserved LPC target must resolve to a pin");
            };
            assert!(!LPC_PIN_MAP.pin(pin).capabilities.available());
        }

        for pin in LPC_PIN_MAP
            .pins()
            .iter()
            .filter(|pin| pin.capabilities.available())
        {
            assert!(pin.capabilities.input());
            assert!(!pin.capabilities.output());
            assert!(!pin.capabilities.pull_up());
        }
    }
}
