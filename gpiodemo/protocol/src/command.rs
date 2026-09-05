use core::fmt;

macro_rules! wire_enum {
    (
        $vis:vis enum $name:ident {
            $($variant:ident => $wire:literal),+ $(,)?
        }
        ; all
    ) => {
        wire_enum!(@define $vis enum $name { $($variant => $wire),+ });

        impl $name {
            $vis const ALL: &'static [Self] = &[$(Self::$variant),+];
        }
    };
    (
        $vis:vis enum $name:ident {
            $($variant:ident => $wire:literal),+ $(,)?
        }
    ) => {
        wire_enum!(@define $vis enum $name { $($variant => $wire),+ });
    };
    (
        @define $vis:vis enum $name:ident {
            $($variant:ident => $wire:literal),+ $(,)?
        }
    ) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        $vis enum $name {
            $($variant),+
        }

        impl AsRef<[u8]> for $name {
            fn as_ref(&self) -> &[u8] {
                match self {
                    $(Self::$variant => $wire),+
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let token = core::str::from_utf8(self.as_ref())
                    .expect("wire enum tokens are ASCII literals");
                f.write_str(token)
            }
        }

        impl TryFrom<&[u8]> for $name {
            type Error = ParseTokenError;

            fn try_from(token: &[u8]) -> Result<Self, Self::Error> {
                match token {
                    $($wire => Ok(Self::$variant)),+,
                    _ => Err(ParseTokenError),
                }
            }
        }
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseTokenError;

impl fmt::Display for ParseTokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid protocol token")
    }
}

impl core::error::Error for ParseTokenError {}

pub const PROTOCOL_VERSION: u16 = 1;

wire_enum! {
    pub enum Command {
        Hello => b"HAI",
        Status => b"HRU",
        Map => b"MAP",
        Direction => b"DIR",
        Get => b"GET",
        Set => b"SET",
        Pullup => b"PLL",
        Listen => b"LSN",
        Query => b"WYD",
        Bye => b"BYE",
        Version => b"VER",
        Help => b"HLP",
    }
    ; all
}

wire_enum! {
    pub enum Direction {
        Input => b"IN",
        Output => b"OUT",
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
    pub const OUTPUT: Self = Self(Self::OUTPUT_BIT);
    pub const INPUT_OUTPUT: Self = Self(Self::INPUT_BIT | Self::OUTPUT_BIT);
    pub const INPUT_PULLUP: Self = Self(Self::INPUT_BIT | Self::PULL_UP_BIT);
    pub const GPIO: Self = Self(Self::INPUT_BIT | Self::OUTPUT_BIT | Self::PULL_UP_BIT);

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

wire_enum! {
    pub enum Query {
        Direction => b"DIR",
        Pullup => b"PLL",
        Listen => b"LSN",
    }
}

wire_enum! {
    pub enum Level {
        Low => b"LOW",
        High => b"HIGH",
    }
}

wire_enum! {
    pub enum Toggle {
        Off => b"OFF",
        On => b"ON",
    }
}

impl From<bool> for Toggle {
    fn from(enabled: bool) -> Self {
        if enabled { Self::On } else { Self::Off }
    }
}

wire_enum! {
    pub enum TargetError {
        Unset => b"UNSET",
        Unavailable => b"UNAVAILABLE",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueryValue {
    Unset,
    Direction(Direction),
    Toggle(Toggle),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request<T> {
    Hello,
    Status,
    Map,
    Direction { target: T, direction: Direction },
    Get { target: T },
    Set { target: T, level: Level },
    Pullup { target: T, state: Toggle },
    Listen { target: T, state: Toggle },
    Query { target: T, what: Query },
    Bye,
    Version,
    Help,
}

impl<T> Request<T> {
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
            Self::Pullup { target, state } => Request::Pullup {
                target: map(target)?,
                state,
            },
            Self::Listen { target, state } => Request::Listen {
                target: map(target)?,
                state,
            },
            Self::Query { target, what } => Request::Query {
                target: map(target)?,
                what,
            },
            Self::Bye => Request::Bye,
            Self::Version => Request::Version,
            Self::Help => Request::Help,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResponseError<T, D> {
    BadPacket,
    Target { target: T, reason: TargetError },
    NoRoute { destination: D },
    RouteBusy { next_hop: D },
    RouteDown { next_hop: D },
}

impl<T, D> ResponseError<T, D> {
    pub fn try_map<T2, D2, E>(
        self,
        map_target: impl FnOnce(T) -> Result<T2, E>,
        map_data: impl FnOnce(D) -> Result<D2, E>,
    ) -> Result<ResponseError<T2, D2>, E> {
        Ok(match self {
            Self::BadPacket => ResponseError::BadPacket,
            Self::Target { target, reason } => ResponseError::Target {
                target: map_target(target)?,
                reason,
            },
            Self::NoRoute { destination } => ResponseError::NoRoute {
                destination: map_data(destination)?,
            },
            Self::RouteBusy { next_hop } => ResponseError::RouteBusy {
                next_hop: map_data(next_hop)?,
            },
            Self::RouteDown { next_hop } => ResponseError::RouteDown {
                next_hop: map_data(next_hop)?,
            },
        })
    }
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
    Version {
        version: u16,
    },
    Help {
        command: Command,
    },
    Error(ResponseError<T, D>),
    Unknown,
    Bye,
}

pub type DecodedRequest<'a> = Request<&'a [u8]>;
pub type DecodedResponse<'a> = Response<&'a [u8], &'a [u8]>;
