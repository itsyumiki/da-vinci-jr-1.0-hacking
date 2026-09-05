#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RequestId(u16);

impl RequestId {
    pub const MIN: u16 = 1;
    pub const MAX: u16 = 999;
    pub const COUNT: usize = (Self::MAX - Self::MIN + 1) as usize;
    pub const FIRST: Self = Self(Self::MIN);

    pub const fn new(raw: u16) -> Option<Self> {
        if raw >= Self::MIN && raw <= Self::MAX {
            Some(Self(raw))
        } else {
            None
        }
    }

    pub const fn get(self) -> u16 {
        self.0
    }

    pub const fn slot(self) -> usize {
        (self.0 - Self::MIN) as usize
    }

    pub const fn next(self) -> Self {
        if self.0 == Self::MAX {
            Self(Self::MIN)
        } else {
            Self(self.0 + 1)
        }
    }
}

impl core::fmt::Display for RequestId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:03}", self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Packet<T> {
    pub id: RequestId,
    pub body: T,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Message<R, B> {
    pub route: R,
    pub packet: Packet<B>,
}

pub type RawMessage<'a> = Message<&'a [u8], &'a [u8]>;

pub(crate) fn valid_route_token(token: &[u8]) -> bool {
    !token.is_empty() && token.iter().all(u8::is_ascii_graphic)
}

pub(crate) fn parse_packet_id(token: &[u8]) -> Option<RequestId> {
    if token.is_empty() || token.len() > 3 || !token.iter().all(u8::is_ascii_digit) {
        return None;
    }
    RequestId::new(core::str::from_utf8(token).ok()?.parse().ok()?)
}
