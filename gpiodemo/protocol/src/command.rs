use core::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseTokenError;

impl fmt::Display for ParseTokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid protocol token")
    }
}

impl core::error::Error for ParseTokenError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Input,
    Output,
}

impl TryFrom<&[u8]> for Direction {
    type Error = ParseTokenError;

    fn try_from(token: &[u8]) -> Result<Self, Self::Error> {
        match token {
            b"IN" => Ok(Self::Input),
            b"OUT" => Ok(Self::Output),
            _ => Err(ParseTokenError),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PinCapabilities(u8);

impl PinCapabilities {
    const INPUT_BIT: u8 = 1 << 0;
    const OUTPUT_BIT: u8 = 1 << 1;
    const PULL_UP_BIT: u8 = 1 << 2;

    pub const NONE: Self = Self(0);
    pub const INPUT: Self = Self(Self::INPUT_BIT);
    pub const INPUT_PULLUP: Self = Self(Self::INPUT_BIT | Self::PULL_UP_BIT);
    pub const GPIO: Self = Self(Self::INPUT_BIT | Self::OUTPUT_BIT | Self::PULL_UP_BIT);

    pub const fn new(input: bool, output: bool, pull_up: bool) -> Self {
        Self(
            (if input { Self::INPUT_BIT } else { 0 })
                | (if output { Self::OUTPUT_BIT } else { 0 })
                | (if pull_up { Self::PULL_UP_BIT } else { 0 }),
        )
    }

    pub const fn from_bits(bits: u8) -> Option<Self> {
        if bits <= Self::GPIO.0 {
            Some(Self(bits))
        } else {
            None
        }
    }

    pub const fn bits(self) -> u8 {
        self.0
    }

    pub const fn available(self) -> bool {
        self.0 != 0
    }

    pub const fn input(self) -> bool {
        self.0 & Self::INPUT_BIT != 0
    }

    pub const fn output(self) -> bool {
        self.0 & Self::OUTPUT_BIT != 0
    }

    pub const fn pull_up(self) -> bool {
        self.0 & Self::PULL_UP_BIT != 0
    }

    pub const fn supports_direction(self, direction: Direction) -> bool {
        match direction {
            Direction::Input => self.input(),
            Direction::Output => self.output(),
        }
    }
}

impl TryFrom<&[u8]> for PinCapabilities {
    type Error = ParseTokenError;

    fn try_from(token: &[u8]) -> Result<Self, Self::Error> {
        let [digit] = token else {
            return Err(ParseTokenError);
        };
        let bits = match digit {
            b'0'..=b'7' => *digit - b'0',
            _ => return Err(ParseTokenError),
        };
        Self::from_bits(bits).ok_or(ParseTokenError)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Query {
    Direction,
    Pullup,
    Listen,
}

impl TryFrom<&[u8]> for Query {
    type Error = ParseTokenError;

    fn try_from(token: &[u8]) -> Result<Self, Self::Error> {
        match token {
            b"DIR" => Ok(Self::Direction),
            b"PLL" => Ok(Self::Pullup),
            b"LSN" => Ok(Self::Listen),
            _ => Err(ParseTokenError),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Low,
    High,
}

impl TryFrom<&[u8]> for Level {
    type Error = ParseTokenError;

    fn try_from(token: &[u8]) -> Result<Self, Self::Error> {
        match token {
            b"LOW" => Ok(Self::Low),
            b"HIGH" => Ok(Self::High),
            _ => Err(ParseTokenError),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueryValue {
    Unset,
    Direction(Direction),
    Enabled(bool),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request<T> {
    Hello,
    Status,
    Map,
    Direction { target: T, direction: Direction },
    Get { target: T },
    Set { target: T, level: Level },
    Pullup { target: T, enabled: bool },
    Listen { target: T, enabled: bool },
    Query { target: T, what: Query },
    Bye,
}

impl<T> Request<T> {
    pub fn map_target<U>(self, map: impl FnOnce(T) -> U) -> Request<U> {
        match self {
            Self::Hello => Request::Hello,
            Self::Status => Request::Status,
            Self::Map => Request::Map,
            Self::Direction { target, direction } => Request::Direction {
                target: map(target),
                direction,
            },
            Self::Get { target } => Request::Get {
                target: map(target),
            },
            Self::Set { target, level } => Request::Set {
                target: map(target),
                level,
            },
            Self::Pullup { target, enabled } => Request::Pullup {
                target: map(target),
                enabled,
            },
            Self::Listen { target, enabled } => Request::Listen {
                target: map(target),
                enabled,
            },
            Self::Query { target, what } => Request::Query {
                target: map(target),
                what,
            },
            Self::Bye => Request::Bye,
        }
    }

    pub fn try_map_target<U, E>(
        self,
        map: impl FnOnce(T) -> Result<U, E>,
    ) -> Result<Request<U>, E> {
        Ok(match self {
            Self::Hello => Request::Hello,
            Self::Status => Request::Status,
            Self::Map => Request::Map,
            Self::Direction { target, direction } => Request::Direction {
                target: map(target)?,
                direction,
            },
            Self::Get { target } => Request::Get {
                target: map(target)?,
            },
            Self::Set { target, level } => Request::Set {
                target: map(target)?,
                level,
            },
            Self::Pullup { target, enabled } => Request::Pullup {
                target: map(target)?,
                enabled,
            },
            Self::Listen { target, enabled } => Request::Listen {
                target: map(target)?,
                enabled,
            },
            Self::Query { target, what } => Request::Query {
                target: map(target)?,
                what,
            },
            Self::Bye => Request::Bye,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetError {
    Unset,
    Unavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResponseError<T, D> {
    BadPacket,
    Target { target: T, reason: TargetError },
    NoRoute { destination: D },
    RouteBusy { next_hop: D },
    RouteDown { next_hop: D },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Response<T, D> {
    Hello,
    Status {
        identity: D,
    },
    MapBank {
        bank: D,
    },
    MapPin {
        target: T,
        package_pin: Option<u16>,
        bank: D,
        bit: u8,
        capabilities: PinCapabilities,
    },
    Ack,
    Value {
        target: T,
        level: Level,
    },
    State {
        target: T,
        what: Query,
        value: QueryValue,
    },
    Error(ResponseError<T, D>),
    Unknown,
    Bye,
}

pub type DecodedRequest<'a> = Request<&'a [u8]>;
pub type DecodedResponse<'a> = Response<&'a [u8], &'a [u8]>;

impl<T, D> Response<T, D> {
    pub fn try_map<T2, D2, E>(
        self,
        map_target: impl FnOnce(T) -> Result<T2, E>,
        map_data: impl FnOnce(D) -> Result<D2, E>,
    ) -> Result<Response<T2, D2>, E> {
        Ok(match self {
            Self::Hello => Response::Hello,
            Self::Status { identity } => Response::Status {
                identity: map_data(identity)?,
            },
            Self::MapBank { bank } => Response::MapBank {
                bank: map_data(bank)?,
            },
            Self::MapPin {
                target,
                package_pin,
                bank,
                bit,
                capabilities,
            } => Response::MapPin {
                target: map_target(target)?,
                package_pin,
                bank: map_data(bank)?,
                bit,
                capabilities,
            },
            Self::Ack => Response::Ack,
            Self::Value { target, level } => Response::Value {
                target: map_target(target)?,
                level,
            },
            Self::State {
                target,
                what,
                value,
            } => Response::State {
                target: map_target(target)?,
                what,
                value,
            },
            Self::Error(ResponseError::BadPacket) => Response::Error(ResponseError::BadPacket),
            Self::Error(ResponseError::Target { target, reason }) => {
                Response::Error(ResponseError::Target {
                    target: map_target(target)?,
                    reason,
                })
            }
            Self::Error(ResponseError::NoRoute { destination }) => {
                Response::Error(ResponseError::NoRoute {
                    destination: map_data(destination)?,
                })
            }
            Self::Error(ResponseError::RouteBusy { next_hop }) => {
                Response::Error(ResponseError::RouteBusy {
                    next_hop: map_data(next_hop)?,
                })
            }
            Self::Error(ResponseError::RouteDown { next_hop }) => {
                Response::Error(ResponseError::RouteDown {
                    next_hop: map_data(next_hop)?,
                })
            }
            Self::Unknown => Response::Unknown,
            Self::Bye => Response::Bye,
        })
    }
}
