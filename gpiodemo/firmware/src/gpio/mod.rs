mod engine;
pub(crate) mod map;

use da_vinci_protocol::Level;

pub(crate) use engine::Firmware;
#[cfg(test)]
pub(crate) use map::Target;
pub use map::{BankId, PinId, PinMap};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PinMode {
    Input,
    InputPullup,
    Output { initial: Level },
}

pub trait GpioHal {
    fn pin_map(&self) -> &'static PinMap;
    fn configure(&mut self, pin: PinId, mode: PinMode);
    fn write(&mut self, pin: PinId, level: Level);
    fn read_bank(&self, bank: BankId) -> u32;
}
