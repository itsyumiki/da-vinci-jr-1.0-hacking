use crate::{
    command::{
        DecodedRequest, DecodedResponse, Direction, Level, Query, QueryValue, Request, Response,
        ResponseError, TargetError,
    },
    framing::MAX_PACKET_LEN,
    message::{Message, Packet, RequestId, parse_packet_id, valid_route_token},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeErrorKind {
    Malformed,
    UnknownCommand,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecodeError {
    pub id: Option<RequestId>,
    pub kind: DecodeErrorKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncodeError {
    OutputTooSmall,
    InvalidRouteToken,
    InvalidTargetToken,
    InvalidIdentity,
}

pub fn decode_message(line: &[u8]) -> Result<Message<'_>, DecodeError> {
    let (id_token, rest) = next_token(line).ok_or(DecodeError {
        id: None,
        kind: DecodeErrorKind::Malformed,
    })?;
    let id = parse_packet_id(id_token).ok_or(DecodeError {
        id: None,
        kind: DecodeErrorKind::Malformed,
    })?;
    let malformed = || DecodeError {
        id: Some(id),
        kind: DecodeErrorKind::Malformed,
    };
    let (route, rest) = next_token(rest).ok_or_else(malformed)?;
    if !valid_route_token(route) {
        return Err(malformed());
    }
    let body = rest.trim_ascii();
    if body.is_empty() {
        return Err(malformed());
    }
    Ok(Message {
        route,
        packet: Packet { id, body },
    })
}

pub fn encode_message(message: Message<'_>, out: &mut [u8]) -> Result<usize, EncodeError> {
    encode_message_with(message.packet.id, message.route, out, |writer| {
        writer.bytes(message.packet.body)
    })
}

pub fn decode_request(packet: Packet<&[u8]>) -> Result<Packet<DecodedRequest<'_>>, DecodeError> {
    let id = packet.id;
    let mut tokens = packet
        .body
        .split(u8::is_ascii_whitespace)
        .filter(|token| !token.is_empty());
    let command = tokens.next().ok_or(DecodeError {
        id: Some(id),
        kind: DecodeErrorKind::Malformed,
    })?;

    let malformed = || DecodeError {
        id: Some(id),
        kind: DecodeErrorKind::Malformed,
    };

    let body = match command {
        b"HAI" if tokens.next().is_none() => Request::Hello,
        b"HRU" if tokens.next().is_none() => Request::Status,
        b"MAP" if tokens.next().is_none() => Request::Map,
        b"BYE" if tokens.next().is_none() => Request::Bye,
        b"DIR" => {
            let target = next_target(&mut tokens, malformed())?;
            let direction: Direction = next_as(&mut tokens, malformed())?;
            expect_suffix(&mut tokens, b"OK?", malformed())?;
            Request::Direction { target, direction }
        }
        b"GET" => {
            let target = next_target(&mut tokens, malformed())?;
            expect_suffix(&mut tokens, b"OK?", malformed())?;
            Request::Get { target }
        }
        b"SET" => {
            let target = next_target(&mut tokens, malformed())?;
            let level: Level = next_as(&mut tokens, malformed())?;
            expect_suffix(&mut tokens, b"OK?", malformed())?;
            Request::Set { target, level }
        }
        b"PLL" => {
            let target = next_target(&mut tokens, malformed())?;
            let enabled =
                parse_enabled(tokens.next().ok_or_else(malformed)?).ok_or_else(malformed)?;
            expect_suffix(&mut tokens, b"OK?", malformed())?;
            Request::Pullup { target, enabled }
        }
        b"LSN" => {
            let target = next_target(&mut tokens, malformed())?;
            let enabled =
                parse_enabled(tokens.next().ok_or_else(malformed)?).ok_or_else(malformed)?;
            expect_suffix(&mut tokens, b"OK?", malformed())?;
            Request::Listen { target, enabled }
        }
        b"WYD" => {
            let target = next_target(&mut tokens, malformed())?;
            let what: Query = next_as(&mut tokens, malformed())?;
            if tokens.next().is_some() {
                return Err(malformed());
            }
            Request::Query { target, what }
        }
        b"HAI" | b"HRU" | b"MAP" | b"BYE" => return Err(malformed()),
        _ => {
            return Err(DecodeError {
                id: Some(id),
                kind: DecodeErrorKind::UnknownCommand,
            });
        }
    };

    Ok(Packet { id, body })
}

pub fn decode_response(packet: Packet<&[u8]>) -> Result<Packet<DecodedResponse<'_>>, DecodeError> {
    let id = packet.id;
    let mut tokens = packet
        .body
        .split(u8::is_ascii_whitespace)
        .filter(|token| !token.is_empty());
    let command = tokens.next().ok_or(DecodeError {
        id: Some(id),
        kind: DecodeErrorKind::Malformed,
    })?;
    let malformed = || DecodeError {
        id: Some(id),
        kind: DecodeErrorKind::Malformed,
    };

    let body = match command {
        b"HII" => {
            expect_suffix(&mut tokens, b"<3", malformed())?;
            Response::Hello
        }
        b"IAM" => {
            let identity = status_identity(packet.body).ok_or_else(malformed)?;
            Response::Status { identity }
        }
        b"MAP" => match tokens.next() {
            Some(b"BANK") => {
                let bank = next_target(&mut tokens, malformed())?;
                expect_suffix(&mut tokens, b"<3", malformed())?;
                Response::MapBank { bank }
            }
            Some(b"PIN") => {
                let target = next_target(&mut tokens, malformed())?;
                let package_pin = match tokens.next().ok_or_else(malformed)? {
                    b"-" => None,
                    token => Some(parse_u16(token).ok_or_else(malformed)?),
                };
                let bank = next_target(&mut tokens, malformed())?;
                let bit = parse_u8(tokens.next().ok_or_else(malformed)?).ok_or_else(malformed)?;
                let capabilities = next_as(&mut tokens, malformed())?;
                expect_suffix(&mut tokens, b"<3", malformed())?;
                Response::MapPin {
                    target,
                    package_pin,
                    bank,
                    bit,
                    capabilities,
                }
            }
            _ => return Err(malformed()),
        },
        b"OKA" => {
            expect_suffix(&mut tokens, b"<3", malformed())?;
            Response::Ack
        }
        b"CYA" => {
            expect_suffix(&mut tokens, b"<3", malformed())?;
            Response::Bye
        }
        b"IDK" => {
            expect_suffix(&mut tokens, b"<3", malformed())?;
            Response::Unknown
        }
        b"UMM" => {
            let error = match tokens.next() {
                Some(b"BAD_PACKET") => ResponseError::BadPacket,
                Some(b"NO_ROUTE") => {
                    let destination = tokens.next().ok_or_else(malformed)?;
                    if !valid_route_token(destination) {
                        return Err(malformed());
                    }
                    ResponseError::NoRoute { destination }
                }
                Some(b"ROUTE_BUSY") => {
                    let next_hop = tokens.next().ok_or_else(malformed)?;
                    if !valid_route_token(next_hop) {
                        return Err(malformed());
                    }
                    ResponseError::RouteBusy { next_hop }
                }
                Some(b"ROUTE_DOWN") => {
                    let next_hop = tokens.next().ok_or_else(malformed)?;
                    if !valid_route_token(next_hop) {
                        return Err(malformed());
                    }
                    ResponseError::RouteDown { next_hop }
                }
                Some(target) if valid_target_token(target) => {
                    let reason = match tokens.next() {
                        Some(b"UNSET") => TargetError::Unset,
                        Some(b"UNAVAILABLE") => TargetError::Unavailable,
                        _ => return Err(malformed()),
                    };
                    ResponseError::Target { target, reason }
                }
                Some(_) => return Err(malformed()),
                None => return Err(malformed()),
            };
            expect_suffix(&mut tokens, b"<3", malformed())?;
            Response::Error(error)
        }
        b"HYG" => {
            let target = next_target(&mut tokens, malformed())?;
            let next = tokens.next().ok_or_else(malformed)?;
            if let Ok(level) = Level::try_from(next) {
                expect_suffix(&mut tokens, b"<3", malformed())?;
                Response::Value { target, level }
            } else {
                let what = Query::try_from(next).map_err(|_| malformed())?;
                let value_token = tokens.next().ok_or_else(malformed)?;
                let value = match what {
                    Query::Direction if value_token == b"UNSET" => QueryValue::Unset,
                    Query::Direction => QueryValue::Direction(
                        Direction::try_from(value_token).map_err(|_| malformed())?,
                    ),
                    Query::Pullup | Query::Listen => match value_token {
                        b"UNSET" => QueryValue::Unset,
                        b"ON" => QueryValue::Enabled(true),
                        b"OFF" => QueryValue::Enabled(false),
                        _ => return Err(malformed()),
                    },
                };
                expect_suffix(&mut tokens, b"<3", malformed())?;
                Response::State {
                    target,
                    what,
                    value,
                }
            }
        }
        _ => {
            return Err(DecodeError {
                id: Some(id),
                kind: DecodeErrorKind::UnknownCommand,
            });
        }
    };

    Ok(Packet { id, body })
}

pub fn encode_request<T: AsRef<[u8]>>(
    packet: Packet<Request<T>>,
    destination: &[u8],
    out: &mut [u8],
) -> Result<usize, EncodeError> {
    encode_message_with(packet.id, destination, out, |writer| {
        encode_request_body(writer, packet.body)
    })
}

fn encode_request_body<T: AsRef<[u8]>>(
    writer: &mut Writer<'_>,
    body: Request<T>,
) -> Result<(), EncodeError> {
    match body {
        Request::Hello => writer.bytes(b"HAI")?,
        Request::Status => writer.bytes(b"HRU")?,
        Request::Map => writer.bytes(b"MAP")?,
        Request::Direction { target, direction } => {
            writer.bytes(b"DIR ")?;
            writer.target(target.as_ref())?;
            writer.bytes(match direction {
                Direction::Input => b" IN OK?",
                Direction::Output => b" OUT OK?",
            })?;
        }
        Request::Get { target } => {
            writer.bytes(b"GET ")?;
            writer.target(target.as_ref())?;
            writer.bytes(b" OK?")?;
        }
        Request::Set { target, level } => {
            writer.bytes(b"SET ")?;
            writer.target(target.as_ref())?;
            writer.bytes(match level {
                Level::Low => b" LOW OK?",
                Level::High => b" HIGH OK?",
            })?;
        }
        Request::Pullup { target, enabled } => {
            writer.bytes(b"PLL ")?;
            writer.target(target.as_ref())?;
            writer.bytes(if enabled { b" ON OK?" } else { b" OFF OK?" })?;
        }
        Request::Listen { target, enabled } => {
            writer.bytes(b"LSN ")?;
            writer.target(target.as_ref())?;
            writer.bytes(if enabled { b" ON OK?" } else { b" OFF OK?" })?;
        }
        Request::Query { target, what } => {
            writer.bytes(b"WYD ")?;
            writer.target(target.as_ref())?;
            writer.bytes(match what {
                Query::Direction => b" DIR",
                Query::Pullup => b" PLL",
                Query::Listen => b" LSN",
            })?;
        }
        Request::Bye => writer.bytes(b"BYE")?,
    }
    Ok(())
}

pub fn encode_response<T: AsRef<[u8]>, D: AsRef<[u8]>>(
    packet: Packet<Response<T, D>>,
    source: &[u8],
    out: &mut [u8],
) -> Result<usize, EncodeError> {
    encode_message_with(packet.id, source, out, |writer| {
        encode_response_body(writer, packet.body)
    })
}

fn encode_response_body<T: AsRef<[u8]>, D: AsRef<[u8]>>(
    writer: &mut Writer<'_>,
    body: Response<T, D>,
) -> Result<(), EncodeError> {
    match body {
        Response::Hello => writer.bytes(b"HII <3")?,
        Response::Status { identity } => {
            if !valid_identity(identity.as_ref()) {
                return Err(EncodeError::InvalidIdentity);
            }
            writer.bytes(b"IAM ")?;
            writer.bytes(identity.as_ref())?;
            writer.bytes(b" <3")?;
        }
        Response::MapBank { bank } => {
            writer.bytes(b"MAP BANK ")?;
            writer.target(bank.as_ref())?;
            writer.bytes(b" <3")?;
        }
        Response::MapPin {
            target,
            package_pin,
            bank,
            bit,
            capabilities,
        } => {
            writer.bytes(b"MAP PIN ")?;
            writer.target(target.as_ref())?;
            writer.bytes(b" ")?;
            if let Some(package_pin) = package_pin {
                writer.decimal(package_pin)?;
            } else {
                writer.bytes(b"-")?;
            }
            writer.bytes(b" ")?;
            writer.target(bank.as_ref())?;
            writer.bytes(b" ")?;
            writer.decimal(u16::from(bit))?;
            writer.bytes(b" ")?;
            writer.bytes(&[b'0' + capabilities.bits()])?;
            writer.bytes(b" <3")?;
        }
        Response::Ack => writer.bytes(b"OKA <3")?,
        Response::Value { target, level } => {
            writer.bytes(b"HYG ")?;
            writer.target(target.as_ref())?;
            writer.bytes(match level {
                Level::Low => b" LOW <3",
                Level::High => b" HIGH <3",
            })?;
        }
        Response::State {
            target,
            what,
            value,
        } => {
            writer.bytes(b"HYG ")?;
            writer.target(target.as_ref())?;
            writer.bytes(match what {
                Query::Direction => b" DIR ",
                Query::Pullup => b" PLL ",
                Query::Listen => b" LSN ",
            })?;
            writer.bytes(match value {
                QueryValue::Unset => b"UNSET",
                QueryValue::Direction(Direction::Input) => b"IN",
                QueryValue::Direction(Direction::Output) => b"OUT",
                QueryValue::Enabled(true) => b"ON",
                QueryValue::Enabled(false) => b"OFF",
            })?;
            writer.bytes(b" <3")?;
        }
        Response::Error(ResponseError::BadPacket) => writer.bytes(b"UMM BAD_PACKET <3")?,
        Response::Error(ResponseError::Target { target, reason }) => {
            writer.bytes(b"UMM ")?;
            writer.target(target.as_ref())?;
            writer.bytes(match reason {
                TargetError::Unset => b" UNSET <3",
                TargetError::Unavailable => b" UNAVAILABLE <3",
            })?;
        }
        Response::Error(ResponseError::NoRoute { destination }) => {
            writer.bytes(b"UMM NO_ROUTE ")?;
            writer.route(destination.as_ref())?;
            writer.bytes(b" <3")?;
        }
        Response::Error(ResponseError::RouteBusy { next_hop }) => {
            writer.bytes(b"UMM ROUTE_BUSY ")?;
            writer.route(next_hop.as_ref())?;
            writer.bytes(b" <3")?;
        }
        Response::Error(ResponseError::RouteDown { next_hop }) => {
            writer.bytes(b"UMM ROUTE_DOWN ")?;
            writer.route(next_hop.as_ref())?;
            writer.bytes(b" <3")?;
        }
        Response::Unknown => writer.bytes(b"IDK <3")?,
        Response::Bye => writer.bytes(b"CYA <3")?,
    }
    Ok(())
}

fn next_as<'a, T>(
    tokens: &mut impl Iterator<Item = &'a [u8]>,
    error: DecodeError,
) -> Result<T, DecodeError>
where
    T: TryFrom<&'a [u8]>,
{
    tokens.next().ok_or(error)?.try_into().map_err(|_| error)
}

fn next_target<'a>(
    tokens: &mut impl Iterator<Item = &'a [u8]>,
    error: DecodeError,
) -> Result<&'a [u8], DecodeError> {
    let target = tokens.next().ok_or(error)?;
    valid_target_token(target).then_some(target).ok_or(error)
}

fn expect_suffix<'a>(
    tokens: &mut impl Iterator<Item = &'a [u8]>,
    suffix: &[u8],
    error: DecodeError,
) -> Result<(), DecodeError> {
    (tokens.next() == Some(suffix) && tokens.next().is_none())
        .then_some(())
        .ok_or(error)
}

fn parse_u8(token: &[u8]) -> Option<u8> {
    u8::try_from(parse_u16(token)?).ok()
}

fn parse_u16(token: &[u8]) -> Option<u16> {
    if token.is_empty() || !token.iter().all(u8::is_ascii_digit) {
        return None;
    }
    token.iter().try_fold(0u16, |value, digit| {
        value.checked_mul(10)?.checked_add(u16::from(*digit - b'0'))
    })
}

fn parse_enabled(token: &[u8]) -> Option<bool> {
    match token {
        b"ON" => Some(true),
        b"OFF" => Some(false),
        _ => None,
    }
}

fn encode_message_with(
    id: RequestId,
    route: &[u8],
    out: &mut [u8],
    write_body: impl FnOnce(&mut Writer<'_>) -> Result<(), EncodeError>,
) -> Result<usize, EncodeError> {
    let capacity = out.len().min(MAX_PACKET_LEN);
    let mut writer = Writer::new(&mut out[..capacity]);
    writer.id(id)?;
    writer.bytes(b" ")?;
    writer.route(route)?;
    writer.bytes(b" ")?;
    write_body(&mut writer)?;
    writer.bytes(b"\n")?;
    Ok(writer.len())
}

fn next_token(input: &[u8]) -> Option<(&[u8], &[u8])> {
    let start = input.iter().position(|byte| !byte.is_ascii_whitespace())?;
    let input = &input[start..];
    let end = input
        .iter()
        .position(u8::is_ascii_whitespace)
        .unwrap_or(input.len());
    Some((&input[..end], &input[end..]))
}

fn valid_target_token(token: &[u8]) -> bool {
    token.first().is_some_and(u8::is_ascii_uppercase)
        && token
            .iter()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || *byte == b'_')
}

fn status_identity(body: &[u8]) -> Option<&[u8]> {
    let identity = body.strip_prefix(b"IAM ")?.strip_suffix(b" <3")?;
    valid_identity(identity).then_some(identity)
}

fn valid_identity(identity: &[u8]) -> bool {
    !identity.is_empty() && identity.split(|byte| *byte == b' ').all(valid_route_token)
}

struct Writer<'a> {
    out: &'a mut [u8],
    len: usize,
}

impl<'a> Writer<'a> {
    fn new(out: &'a mut [u8]) -> Self {
        Self { out, len: 0 }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn bytes(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
        let end = self
            .len
            .checked_add(bytes.len())
            .filter(|end| *end <= self.out.len())
            .ok_or(EncodeError::OutputTooSmall)?;
        self.out[self.len..end].copy_from_slice(bytes);
        self.len = end;
        Ok(())
    }

    fn id(&mut self, value: RequestId) -> Result<(), EncodeError> {
        self.decimal3(value.get())
    }

    fn route(&mut self, route: &[u8]) -> Result<(), EncodeError> {
        if !valid_route_token(route) {
            return Err(EncodeError::InvalidRouteToken);
        }
        self.bytes(route)
    }

    fn target(&mut self, target: &[u8]) -> Result<(), EncodeError> {
        if !valid_target_token(target) {
            return Err(EncodeError::InvalidTargetToken);
        }
        self.bytes(target)
    }

    fn decimal(&mut self, value: u16) -> Result<(), EncodeError> {
        let mut digits = [0u8; 5];
        let mut value = value;
        let mut start = digits.len();
        loop {
            start -= 1;
            digits[start] = b'0' + (value % 10) as u8;
            value /= 10;
            if value == 0 {
                return self.bytes(&digits[start..]);
            }
        }
    }

    fn decimal3(&mut self, value: u16) -> Result<(), EncodeError> {
        self.bytes(&[
            b'0' + ((value / 100) % 10) as u8,
            b'0' + ((value / 10) % 10) as u8,
            b'0' + (value % 10) as u8,
        ])
    }
}
