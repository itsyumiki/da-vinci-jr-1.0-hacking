pub(crate) use da_vinci_protocol::PinCapabilities as Capabilities;

pub(crate) const MAX_PINS: usize = 128;
pub(crate) const MAX_BANKS: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PinId(u8);

impl PinId {
    pub(crate) const fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BankId(u8);

impl BankId {
    #[cfg(any(test, target_arch = "arm"))]
    pub(crate) const fn new(index: u8) -> Self {
        Self(index)
    }

    pub(crate) const fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PinInfo {
    pub(crate) token: &'static str,
    pub(crate) package_pin: Option<u16>,
    pub(crate) bank: BankId,
    pub(crate) bit: u8,
    pub(crate) capabilities: Capabilities,
}

impl PinInfo {
    #[cfg(any(test, target_arch = "arm"))]
    pub(crate) const fn new(
        token: &'static str,
        package_pin: Option<u16>,
        bank: BankId,
        bit: u8,
        capabilities: Capabilities,
    ) -> Self {
        Self {
            token,
            package_pin,
            bank,
            bit,
            capabilities,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Target {
    Pin(PinId),
    Bank(BankId),
    All,
}

pub struct PinMap {
    banks: &'static [&'static str],
    pins: &'static [PinInfo],
}

impl PinMap {
    #[cfg(any(test, target_arch = "arm"))]
    pub(crate) const fn new(banks: &'static [&'static str], pins: &'static [PinInfo]) -> Self {
        assert!(banks.len() <= MAX_BANKS);
        assert!(pins.len() <= MAX_PINS);
        assert!(banks.len() <= u8::MAX as usize);
        assert!(pins.len() <= u8::MAX as usize);
        Self { banks, pins }
    }

    pub(crate) const fn banks(&self) -> &'static [&'static str] {
        self.banks
    }

    pub(crate) const fn pins(&self) -> &'static [PinInfo] {
        self.pins
    }

    pub(crate) fn bank(&self, id: BankId) -> &'static str {
        self.banks[id.index()]
    }

    pub(crate) fn pin(&self, id: PinId) -> &'static PinInfo {
        &self.pins[id.index()]
    }

    pub(crate) fn pin_id(&self, index: usize) -> PinId {
        assert!(index < self.pins.len(), "pin index belongs to this PinMap");
        PinId(u8::try_from(index).expect("PinMap pin index fits PinId"))
    }

    fn bank_id(&self, index: usize) -> BankId {
        assert!(
            index < self.banks.len(),
            "bank index belongs to this PinMap"
        );
        BankId(u8::try_from(index).expect("PinMap bank index fits BankId"))
    }

    pub(crate) fn pin_ids(&self) -> impl Iterator<Item = PinId> + '_ {
        (0..self.pins.len()).map(|index| self.pin_id(index))
    }

    pub(crate) fn resolve(&self, token: &[u8]) -> Option<Target> {
        if token == b"ALL" {
            return Some(Target::All);
        }
        if let Some(index) = self.banks.iter().position(|bank| bank.as_bytes() == token) {
            return Some(Target::Bank(self.bank_id(index)));
        }
        self.pins
            .iter()
            .position(|pin| pin.token.as_bytes() == token)
            .map(|index| Target::Pin(self.pin_id(index)))
    }

    pub(crate) fn pins_for(&self, target: Target) -> impl Iterator<Item = PinId> + '_ {
        self.pins
            .iter()
            .enumerate()
            .filter_map(move |(index, pin)| {
                let id = self.pin_id(index);
                match target {
                    Target::Pin(target) if target == id => Some(id),
                    Target::Bank(bank) if pin.bank == bank => Some(id),
                    Target::All => Some(id),
                    _ => None,
                }
            })
    }
}
