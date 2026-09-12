#[cfg(any(test, all(target_arch = "arm", feature = "sam4e8e")))]
use crate::gpio::map::{BankId, Capabilities, PinInfo, PinMap};

#[cfg(all(target_arch = "arm", feature = "sam4e8e"))]
use atsam4_hal::{
    hal::serial::{Read, Write},
    pac,
    serial::Uart1,
};
#[cfg(all(target_arch = "arm", feature = "sam4e8e"))]
use da_vinci_protocol::Level;

#[cfg(all(target_arch = "arm", feature = "sam4e8e"))]
use crate::{
    gpio::{GpioHal, PinId, PinMode},
    transport::{ByteError, NonBlockingBytes},
};

#[cfg(any(test, all(target_arch = "arm", feature = "sam4e8e")))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SamBank {
    A,
    B,
    C,
    D,
    E,
}

#[cfg(any(test, all(target_arch = "arm", feature = "sam4e8e")))]
impl SamBank {
    pub(crate) const fn id(self) -> BankId {
        BankId::new(match self {
            Self::A => 0,
            Self::B => 1,
            Self::C => 2,
            Self::D => 3,
            Self::E => 4,
        })
    }

    #[cfg(all(target_arch = "arm", feature = "sam4e8e"))]
    pub(crate) fn from_id(id: BankId) -> Option<Self> {
        match id.index() {
            0 => Some(Self::A),
            1 => Some(Self::B),
            2 => Some(Self::C),
            3 => Some(Self::D),
            4 => Some(Self::E),
            _ => None,
        }
    }
}

#[cfg(any(test, all(target_arch = "arm", feature = "sam4e8e")))]
pub(crate) const BANK_A: BankId = SamBank::A.id();
#[cfg(any(test, all(target_arch = "arm", feature = "sam4e8e")))]
pub(crate) const BANK_B: BankId = SamBank::B.id();
#[cfg(any(test, all(target_arch = "arm", feature = "sam4e8e")))]
pub(crate) const BANK_C: BankId = SamBank::C.id();
#[cfg(any(test, all(target_arch = "arm", feature = "sam4e8e")))]
pub(crate) const BANK_D: BankId = SamBank::D.id();
#[cfg(any(test, all(target_arch = "arm", feature = "sam4e8e")))]
pub(crate) const BANK_E: BankId = SamBank::E.id();

pub const SAM_IDENTITY: &[u8] = b"SAM4E8E GPIO";

#[cfg(any(test, all(target_arch = "arm", feature = "sam4e8e")))]
static BANKS: [&str; 5] = ["PIOA", "PIOB", "PIOC", "PIOD", "PIOE"];

#[cfg(any(test, all(target_arch = "arm", feature = "sam4e8e")))]
static PINS: [PinInfo; 117] = [
    PinInfo::new("PA00", Some(102), BANK_A, 0, Capabilities::GPIO),
    PinInfo::new("PA01", Some(99), BANK_A, 1, Capabilities::GPIO),
    PinInfo::new("PA02", Some(93), BANK_A, 2, Capabilities::GPIO),
    PinInfo::new("PA03", Some(91), BANK_A, 3, Capabilities::GPIO),
    PinInfo::new("PA04", Some(77), BANK_A, 4, Capabilities::GPIO),
    PinInfo::new("PA05", Some(73), BANK_A, 5, Capabilities::NONE),
    PinInfo::new("PA06", Some(114), BANK_A, 6, Capabilities::NONE),
    PinInfo::new("PA07", Some(35), BANK_A, 7, Capabilities::GPIO),
    PinInfo::new("PA08", Some(36), BANK_A, 8, Capabilities::GPIO),
    PinInfo::new("PA09", Some(75), BANK_A, 9, Capabilities::GPIO),
    PinInfo::new("PA10", Some(66), BANK_A, 10, Capabilities::GPIO),
    PinInfo::new("PA11", Some(64), BANK_A, 11, Capabilities::GPIO),
    PinInfo::new("PA12", Some(68), BANK_A, 12, Capabilities::GPIO),
    PinInfo::new("PA13", Some(42), BANK_A, 13, Capabilities::GPIO),
    PinInfo::new("PA14", Some(51), BANK_A, 14, Capabilities::GPIO),
    PinInfo::new("PA15", Some(49), BANK_A, 15, Capabilities::GPIO),
    PinInfo::new("PA16", Some(45), BANK_A, 16, Capabilities::GPIO),
    PinInfo::new("PA17", Some(25), BANK_A, 17, Capabilities::GPIO),
    PinInfo::new("PA18", Some(24), BANK_A, 18, Capabilities::GPIO),
    PinInfo::new("PA19", Some(23), BANK_A, 19, Capabilities::GPIO),
    PinInfo::new("PA20", Some(22), BANK_A, 20, Capabilities::GPIO),
    PinInfo::new("PA21", Some(32), BANK_A, 21, Capabilities::GPIO),
    PinInfo::new("PA22", Some(37), BANK_A, 22, Capabilities::GPIO),
    PinInfo::new("PA23", Some(46), BANK_A, 23, Capabilities::GPIO),
    PinInfo::new("PA24", Some(56), BANK_A, 24, Capabilities::GPIO),
    PinInfo::new("PA25", Some(59), BANK_A, 25, Capabilities::GPIO),
    PinInfo::new("PA26", Some(62), BANK_A, 26, Capabilities::GPIO),
    PinInfo::new("PA27", Some(70), BANK_A, 27, Capabilities::GPIO),
    PinInfo::new("PA28", Some(112), BANK_A, 28, Capabilities::GPIO),
    PinInfo::new("PA29", Some(129), BANK_A, 29, Capabilities::GPIO),
    PinInfo::new("PA30", Some(116), BANK_A, 30, Capabilities::GPIO),
    PinInfo::new("PA31", Some(118), BANK_A, 31, Capabilities::GPIO),
    PinInfo::new("PB00", Some(21), BANK_B, 0, Capabilities::GPIO),
    PinInfo::new("PB01", Some(20), BANK_B, 1, Capabilities::GPIO),
    PinInfo::new("PB02", Some(26), BANK_B, 2, Capabilities::GPIO),
    PinInfo::new("PB03", Some(31), BANK_B, 3, Capabilities::GPIO),
    PinInfo::new("PB04", Some(105), BANK_B, 4, Capabilities::GPIO),
    PinInfo::new("PB05", Some(109), BANK_B, 5, Capabilities::GPIO),
    PinInfo::new("PB06", Some(79), BANK_B, 6, Capabilities::GPIO),
    PinInfo::new("PB07", Some(89), BANK_B, 7, Capabilities::GPIO),
    PinInfo::new("PB08", Some(141), BANK_B, 8, Capabilities::NONE),
    PinInfo::new("PB09", Some(142), BANK_B, 9, Capabilities::NONE),
    PinInfo::new("PB10", Some(136), BANK_B, 10, Capabilities::NONE),
    PinInfo::new("PB11", Some(137), BANK_B, 11, Capabilities::NONE),
    PinInfo::new("PB12", Some(87), BANK_B, 12, Capabilities::GPIO),
    PinInfo::new("PB13", Some(144), BANK_B, 13, Capabilities::GPIO),
    PinInfo::new("PB14", Some(140), BANK_B, 14, Capabilities::GPIO),
    PinInfo::new("PC00", Some(11), BANK_C, 0, Capabilities::GPIO),
    PinInfo::new("PC01", Some(38), BANK_C, 1, Capabilities::GPIO),
    PinInfo::new("PC02", Some(39), BANK_C, 2, Capabilities::GPIO),
    PinInfo::new("PC03", Some(40), BANK_C, 3, Capabilities::GPIO),
    PinInfo::new("PC04", Some(41), BANK_C, 4, Capabilities::GPIO),
    PinInfo::new("PC05", Some(58), BANK_C, 5, Capabilities::GPIO),
    PinInfo::new("PC06", Some(54), BANK_C, 6, Capabilities::GPIO),
    PinInfo::new("PC07", Some(48), BANK_C, 7, Capabilities::GPIO),
    PinInfo::new("PC08", Some(82), BANK_C, 8, Capabilities::GPIO),
    PinInfo::new("PC09", Some(86), BANK_C, 9, Capabilities::GPIO),
    PinInfo::new("PC10", Some(90), BANK_C, 10, Capabilities::GPIO),
    PinInfo::new("PC11", Some(94), BANK_C, 11, Capabilities::GPIO),
    PinInfo::new("PC12", Some(17), BANK_C, 12, Capabilities::GPIO),
    PinInfo::new("PC13", Some(19), BANK_C, 13, Capabilities::GPIO),
    PinInfo::new("PC14", Some(97), BANK_C, 14, Capabilities::GPIO),
    PinInfo::new("PC15", Some(18), BANK_C, 15, Capabilities::GPIO),
    PinInfo::new("PC16", Some(100), BANK_C, 16, Capabilities::GPIO),
    PinInfo::new("PC17", Some(103), BANK_C, 17, Capabilities::GPIO),
    PinInfo::new("PC18", Some(111), BANK_C, 18, Capabilities::GPIO),
    PinInfo::new("PC19", Some(117), BANK_C, 19, Capabilities::GPIO),
    PinInfo::new("PC20", Some(120), BANK_C, 20, Capabilities::GPIO),
    PinInfo::new("PC21", Some(122), BANK_C, 21, Capabilities::GPIO),
    PinInfo::new("PC22", Some(124), BANK_C, 22, Capabilities::GPIO),
    PinInfo::new("PC23", Some(127), BANK_C, 23, Capabilities::GPIO),
    PinInfo::new("PC24", Some(130), BANK_C, 24, Capabilities::GPIO),
    PinInfo::new("PC25", Some(133), BANK_C, 25, Capabilities::GPIO),
    PinInfo::new("PC26", Some(13), BANK_C, 26, Capabilities::GPIO),
    PinInfo::new("PC27", Some(12), BANK_C, 27, Capabilities::GPIO),
    PinInfo::new("PC28", Some(76), BANK_C, 28, Capabilities::GPIO),
    PinInfo::new("PC29", Some(16), BANK_C, 29, Capabilities::GPIO),
    PinInfo::new("PC30", Some(15), BANK_C, 30, Capabilities::GPIO),
    PinInfo::new("PC31", Some(14), BANK_C, 31, Capabilities::GPIO),
    PinInfo::new("PD00", Some(1), BANK_D, 0, Capabilities::GPIO),
    PinInfo::new("PD01", Some(132), BANK_D, 1, Capabilities::GPIO),
    PinInfo::new("PD02", Some(131), BANK_D, 2, Capabilities::GPIO),
    PinInfo::new("PD03", Some(128), BANK_D, 3, Capabilities::GPIO),
    PinInfo::new("PD04", Some(126), BANK_D, 4, Capabilities::GPIO),
    PinInfo::new("PD05", Some(125), BANK_D, 5, Capabilities::GPIO),
    PinInfo::new("PD06", Some(121), BANK_D, 6, Capabilities::GPIO),
    PinInfo::new("PD07", Some(119), BANK_D, 7, Capabilities::GPIO),
    PinInfo::new("PD08", Some(113), BANK_D, 8, Capabilities::GPIO),
    PinInfo::new("PD09", Some(110), BANK_D, 9, Capabilities::GPIO),
    PinInfo::new("PD10", Some(101), BANK_D, 10, Capabilities::GPIO),
    PinInfo::new("PD11", Some(98), BANK_D, 11, Capabilities::GPIO),
    PinInfo::new("PD12", Some(92), BANK_D, 12, Capabilities::GPIO),
    PinInfo::new("PD13", Some(88), BANK_D, 13, Capabilities::GPIO),
    PinInfo::new("PD14", Some(84), BANK_D, 14, Capabilities::GPIO),
    PinInfo::new("PD15", Some(106), BANK_D, 15, Capabilities::GPIO),
    PinInfo::new("PD16", Some(78), BANK_D, 16, Capabilities::GPIO),
    PinInfo::new("PD17", Some(74), BANK_D, 17, Capabilities::GPIO),
    PinInfo::new("PD18", Some(69), BANK_D, 18, Capabilities::GPIO),
    PinInfo::new("PD19", Some(67), BANK_D, 19, Capabilities::GPIO),
    PinInfo::new("PD20", Some(65), BANK_D, 20, Capabilities::GPIO),
    PinInfo::new("PD21", Some(63), BANK_D, 21, Capabilities::GPIO),
    PinInfo::new("PD22", Some(60), BANK_D, 22, Capabilities::GPIO),
    PinInfo::new("PD23", Some(57), BANK_D, 23, Capabilities::GPIO),
    PinInfo::new("PD24", Some(55), BANK_D, 24, Capabilities::GPIO),
    PinInfo::new("PD25", Some(52), BANK_D, 25, Capabilities::GPIO),
    PinInfo::new("PD26", Some(53), BANK_D, 26, Capabilities::GPIO),
    PinInfo::new("PD27", Some(47), BANK_D, 27, Capabilities::GPIO),
    PinInfo::new("PD28", Some(71), BANK_D, 28, Capabilities::GPIO),
    PinInfo::new("PD29", Some(108), BANK_D, 29, Capabilities::GPIO),
    PinInfo::new("PD30", Some(34), BANK_D, 30, Capabilities::GPIO),
    PinInfo::new("PD31", Some(2), BANK_D, 31, Capabilities::GPIO),
    PinInfo::new("PE00", Some(4), BANK_E, 0, Capabilities::GPIO),
    PinInfo::new("PE01", Some(6), BANK_E, 1, Capabilities::GPIO),
    PinInfo::new("PE02", Some(7), BANK_E, 2, Capabilities::GPIO),
    PinInfo::new("PE03", Some(10), BANK_E, 3, Capabilities::GPIO),
    PinInfo::new("PE04", Some(27), BANK_E, 4, Capabilities::GPIO),
    PinInfo::new("PE05", Some(28), BANK_E, 5, Capabilities::GPIO),
];

#[cfg(any(test, all(target_arch = "arm", feature = "sam4e8e")))]
pub(crate) static SAM_PIN_MAP: PinMap = PinMap::new(&BANKS, &PINS);

#[cfg(all(target_arch = "arm", feature = "sam4e8e"))]
pub struct SamGpio;

#[cfg(all(target_arch = "arm", feature = "sam4e8e"))]
macro_rules! with_pin {
    ($pin:expr, |$port:ident, $mask:ident| $body:block) => {{
        let info = SAM_PIN_MAP.pin($pin);
        let $mask = 1u32 << info.bit;
        // SAFETY: Firmware only supplies IDs from SAM_PIN_MAP. Reserved clock/USB pins do not
        // reach this adapter, and the firmware loop is single-threaded, so the MMIO registers are
        // not aliased by another GPIO adapter.
        unsafe {
            match SamBank::from_id(info.bank).expect("SAM pin map contains only PIOA-E") {
                SamBank::A => {
                    let $port = &*pac::PIOA::ptr();
                    $body
                }
                SamBank::B => {
                    let $port = &*pac::PIOB::ptr();
                    $body
                }
                SamBank::C => {
                    let $port = &*pac::PIOC::ptr();
                    $body
                }
                SamBank::D => {
                    let $port = &*pac::PIOD::ptr();
                    $body
                }
                SamBank::E => {
                    let $port = &*pac::PIOE::ptr();
                    $body
                }
            }
        }
    }};
}

#[cfg(all(target_arch = "arm", feature = "sam4e8e"))]
impl GpioHal for SamGpio {
    fn pin_map(&self) -> &'static PinMap {
        &SAM_PIN_MAP
    }

    fn configure(&mut self, pin: PinId, mode: PinMode) {
        with_pin!(pin, |port, mask| {
            match mode {
                PinMode::Input => {
                    port.pudr.write_with_zero(|w| w.bits(mask));
                    port.ppddr.write_with_zero(|w| w.bits(mask));
                    port.odr.write_with_zero(|w| w.bits(mask));
                }
                PinMode::InputPullup => {
                    port.ppddr.write_with_zero(|w| w.bits(mask));
                    port.puer.write_with_zero(|w| w.bits(mask));
                    port.odr.write_with_zero(|w| w.bits(mask));
                }
                PinMode::Output { initial } => {
                    port.pudr.write_with_zero(|w| w.bits(mask));
                    port.ppddr.write_with_zero(|w| w.bits(mask));
                    if initial == Level::High {
                        port.sodr.write_with_zero(|w| w.bits(mask));
                    } else {
                        port.codr.write_with_zero(|w| w.bits(mask));
                    }
                    port.oer.write_with_zero(|w| w.bits(mask));
                    port.ower.write_with_zero(|w| w.bits(mask));
                }
            }
            port.per.write_with_zero(|w| w.bits(mask));
        });
    }

    fn write(&mut self, pin: PinId, level: Level) {
        with_pin!(pin, |port, mask| {
            if level == Level::High {
                port.sodr.write_with_zero(|w| w.bits(mask));
            } else {
                port.codr.write_with_zero(|w| w.bits(mask));
            }
        });
    }

    fn read_bank(&self, bank: BankId) -> u32 {
        // SAFETY: reading PDSR is side-effect free and SamGpio is the sole GPIO register adapter.
        unsafe {
            match SamBank::from_id(bank).expect("SAM pin map contains only PIOA-E") {
                SamBank::A => (&*pac::PIOA::ptr()).pdsr.read().bits(),
                SamBank::B => (&*pac::PIOB::ptr()).pdsr.read().bits(),
                SamBank::C => (&*pac::PIOC::ptr()).pdsr.read().bits(),
                SamBank::D => (&*pac::PIOD::ptr()).pdsr.read().bits(),
                SamBank::E => (&*pac::PIOE::ptr()).pdsr.read().bits(),
            }
        }
    }
}

#[cfg(all(target_arch = "arm", feature = "sam4e8e"))]
pub struct SamUartBytes(Uart1);

#[cfg(all(target_arch = "arm", feature = "sam4e8e"))]
impl SamUartBytes {
    pub const fn new(uart: Uart1) -> Self {
        Self(uart)
    }
}

#[cfg(all(target_arch = "arm", feature = "sam4e8e"))]
impl NonBlockingBytes for SamUartBytes {
    fn try_read(&mut self, out: &mut [u8]) -> Result<usize, ByteError> {
        let mut read = 0;
        while read < out.len() {
            match self.0.read() {
                Ok(byte) => {
                    out[read] = byte;
                    read += 1;
                }
                Err(nb::Error::WouldBlock) => break,
                Err(nb::Error::Other(_)) => return Err(ByteError::Down),
            }
        }
        if read == 0 {
            Err(ByteError::WouldBlock)
        } else {
            Ok(read)
        }
    }

    fn try_write(&mut self, bytes: &[u8]) -> Result<usize, ByteError> {
        let mut written = 0;
        while written < bytes.len() {
            match self.0.write(bytes[written]) {
                Ok(()) => written += 1,
                Err(nb::Error::WouldBlock) => break,
                Err(nb::Error::Other(_)) => return Err(ByteError::Down),
            }
        }
        if written == 0 {
            Err(ByteError::WouldBlock)
        } else {
            Ok(written)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpio::Target;

    #[test]
    fn map_preserves_native_names_package_pins_and_reservations() {
        assert_eq!(SAM_PIN_MAP.banks().len(), 5);
        assert_eq!(SAM_PIN_MAP.pins().len(), 117);
        assert_eq!(SAM_PIN_MAP.bank(BANK_C), "PIOC");

        let Target::Pin(pb12) = SAM_PIN_MAP.resolve(b"PB12").unwrap() else {
            panic!("PB12 must resolve to a pin");
        };
        assert_eq!(SAM_PIN_MAP.pin(pb12).package_pin, Some(87));

        for token in [b"PA05".as_slice(), b"PA06"] {
            let Target::Pin(pin) = SAM_PIN_MAP.resolve(token).unwrap() else {
                panic!("reserved SAM UART target must still be present in metadata");
            };
            assert!(!SAM_PIN_MAP.pin(pin).capabilities.available());
            assert_eq!(SAM_PIN_MAP.pin(pin).bank, BANK_A);
        }

        for token in [b"PB08".as_slice(), b"PB09", b"PB10", b"PB11"] {
            let Target::Pin(pin) = SAM_PIN_MAP.resolve(token).unwrap() else {
                panic!("reserved SAM target must still be present in metadata");
            };
            assert!(!SAM_PIN_MAP.pin(pin).capabilities.available());
            assert_eq!(SAM_PIN_MAP.pin(pin).bank, BANK_B);
        }
        assert_eq!(SAM_IDENTITY, b"SAM4E8E GPIO");
    }
}
