use std::{array, fmt};

use da_vinci_protocol::{
    Command, DecodeError, Direction, Frame, Level, MAX_PACKET_LEN, Message, Packet,
    PinCapabilities, Query, QueryValue, Request as ProtocolRequest, RequestId,
    ResponseError as ProtocolResponseError, Toggle,
};

use crate::io::{
    IoEvent, ListenerKey, ListenerPin, ListenerRoute, OwnedResponse, SerialIo, WireResponse,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RouteKey(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct BankKey {
    route: RouteKey,
    index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PinKey {
    route: RouteKey,
    index: usize,
}

impl From<PinKey> for ListenerKey {
    fn from(pin: PinKey) -> Self {
        Self {
            route: pin.route.0,
            pin: pin.index,
        }
    }
}

impl From<ListenerKey> for PinKey {
    fn from(key: ListenerKey) -> Self {
        Self {
            route: RouteKey(key.route),
            index: key.pin,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Target {
    Pin(PinKey),
    Bank(BankKey),
    All,
}

pub(super) type Request = ProtocolRequest<Target>;
pub(super) type ResponseError = ProtocolResponseError<PinKey, String>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PinInfo {
    pub(super) token: String,
    pub(super) package_pin: Option<u16>,
    pub(super) bank: BankKey,
    pub(super) bit: u8,
    pub(super) capabilities: PinCapabilities,
}

impl fmt::Display for PinInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.package_pin {
            Some(package_pin) => write!(f, "{} ({package_pin})", self.token),
            None => f.write_str(&self.token),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Event {
    Connected(String),
    Disconnected(Option<String>),
    Received {
        line: String,
        event: Result<DeviceEvent, String>,
    },
    ListenerValues(Vec<ListenerValue>),
    IoError(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ListenerValue {
    line: String,
    id: RequestId,
    pub(super) pin: PinKey,
    pub(super) level: Level,
    pub(super) coalesced: u32,
}

impl ListenerValue {
    pub(super) fn line(&self) -> &str {
        &self.line
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum DeviceEvent {
    Hello {
        route: RouteKey,
    },
    Status {
        route: RouteKey,
        identity: String,
    },
    Version {
        route: RouteKey,
        version: u16,
    },
    Help {
        route: RouteKey,
        command: Command,
    },
    MapReady {
        route: RouteKey,
    },
    Ack {
        route: RouteKey,
        sent: Option<String>,
    },
    PinValue {
        pin: PinKey,
        level: Level,
    },
    PinState {
        pin: PinKey,
        what: Query,
        value: QueryValue,
    },
    DeviceError {
        route: RouteKey,
        source: String,
        error: ResponseError,
    },
    Unknown {
        route: RouteKey,
    },
    Bye {
        route: RouteKey,
    },
    Untracked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RequestLifetime {
    OneShot,
    StreamUntilAck,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Mode {
    Input,
    InputPullup,
    Output,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Input => "INPUT",
            Self::InputPullup => "IN_PULLUP",
            Self::Output => "OUTPUT",
        })
    }
}

impl Mode {
    pub(super) const ALL: &'static [Self] = &[Self::Input, Self::InputPullup, Self::Output];
    const INPUT_ONLY: &'static [Self] = &[Self::Input];
    const OUTPUT_ONLY: &'static [Self] = &[Self::Output];
    const INPUT_WITH_PULLUP: &'static [Self] = &[Self::Input, Self::InputPullup];
    const INPUT_OUTPUT: &'static [Self] = &[Self::Input, Self::Output];

    pub(super) const fn is_input(self) -> bool {
        matches!(self, Self::Input | Self::InputPullup)
    }

    pub(super) const fn available_for(capabilities: PinCapabilities) -> &'static [Self] {
        Self::available(
            capabilities.input(),
            capabilities.output(),
            capabilities.pull_up(),
        )
    }

    pub(super) fn available_for_any(
        capabilities: impl IntoIterator<Item = PinCapabilities>,
    ) -> &'static [Self] {
        let mut input = false;
        let mut output = false;
        let mut pull_up = false;
        for capabilities in capabilities {
            input |= Self::Input.supported_by(capabilities);
            output |= Self::Output.supported_by(capabilities);
            pull_up |= Self::InputPullup.supported_by(capabilities);
        }
        Self::available(input, output, pull_up)
    }

    pub(super) const fn supported_by(self, capabilities: PinCapabilities) -> bool {
        match self {
            Self::Input => capabilities.input(),
            Self::InputPullup => capabilities.input() && capabilities.pull_up(),
            Self::Output => capabilities.output(),
        }
    }

    const fn available(input: bool, output: bool, pull_up: bool) -> &'static [Self] {
        match (input, output, pull_up) {
            (false, false, _) => &[],
            (true, false, false) => Self::INPUT_ONLY,
            (false, true, _) => Self::OUTPUT_ONLY,
            (true, false, true) => Self::INPUT_WITH_PULLUP,
            (true, true, false) => Self::INPUT_OUTPUT,
            (true, true, true) => Self::ALL,
        }
    }

    const fn direction(self) -> Direction {
        if matches!(self, Self::Output) {
            Direction::Output
        } else {
            Direction::Input
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ListenerState {
    Off,
    Enabling {
        request_id: RequestId,
    },
    On {
        stream_id: RequestId,
    },
    Disabling {
        request_id: RequestId,
        stream_id: RequestId,
    },
}

impl ListenerState {
    pub(super) const fn is_pending(self) -> bool {
        matches!(self, Self::Enabling { .. } | Self::Disabling { .. })
    }

    pub(super) const fn stream_id(self) -> Option<RequestId> {
        match self {
            Self::On { stream_id } | Self::Disabling { stream_id, .. } => Some(stream_id),
            Self::Off | Self::Enabling { .. } => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PinState {
    pub(super) mode: Option<Mode>,
    pub(super) target_mode: Option<Mode>,
    pub(super) level: Option<Level>,
    pub(super) listener: ListenerState,
    pub(super) value_pending: bool,
}

impl PinState {
    pub(super) const UNSET: Self = Self {
        mode: None,
        target_mode: None,
        level: None,
        listener: ListenerState::Off,
        value_pending: false,
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Pending {
    route: RouteKey,
    request: Request,
}

#[derive(Clone, Debug)]
struct RoutePin {
    info: PinInfo,
    state: PinState,
}

#[derive(Clone, Debug, Default)]
struct RouteMap {
    banks: Vec<String>,
    pins: Vec<RoutePin>,
}

#[derive(Clone, Debug)]
struct MapBuilder {
    route: RouteKey,
    banks: Vec<String>,
    pins: Vec<PinInfo>,
}

impl MapBuilder {
    fn new(route: RouteKey) -> Self {
        Self {
            route,
            banks: Vec::new(),
            pins: Vec::new(),
        }
    }

    fn bank(&mut self, token: String) -> Result<(), String> {
        if self.banks.iter().any(|bank| bank == &token) {
            return Err(format!("Duplicate MAP bank {token}"));
        }
        self.banks.push(token);
        Ok(())
    }

    fn pin(
        &mut self,
        token: String,
        package_pin: Option<u16>,
        bank_token: String,
        bit: u8,
        capabilities: PinCapabilities,
    ) -> Result<(), String> {
        if self.pins.iter().any(|pin| pin.token == token) {
            return Err(format!("Duplicate MAP pin {token}"));
        }
        let Some(bank_index) = self.banks.iter().position(|bank| bank == &bank_token) else {
            return Err(format!(
                "MAP pin {token} references unknown bank {bank_token}"
            ));
        };
        if self
            .pins
            .iter()
            .any(|pin| pin.bank.index == bank_index && pin.bit == bit)
        {
            return Err(format!("Duplicate MAP bank bit {bank_token}:{bit}"));
        }
        self.pins.push(PinInfo {
            token,
            package_pin,
            bank: BankKey {
                route: self.route,
                index: bank_index,
            },
            bit,
            capabilities,
        });
        Ok(())
    }

    fn finish(self) -> RouteMap {
        RouteMap {
            banks: self.banks,
            pins: self
                .pins
                .into_iter()
                .map(|info| RoutePin {
                    info,
                    state: PinState::UNSET,
                })
                .collect(),
        }
    }
}

struct RouteState {
    name: String,
    map: Option<RouteMap>,
    discovery: Option<MapBuilder>,
}

impl RouteState {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            map: None,
            discovery: None,
        }
    }
}

pub(super) struct DeviceSession {
    next_id: RequestId,
    pending: [Option<Pending>; RequestId::COUNT],
    routes: Vec<RouteState>,
    io: SerialIo,
}

impl DeviceSession {
    pub(super) fn spawn(route_names: &[&str]) -> Self {
        Self {
            next_id: RequestId::FIRST,
            pending: array::from_fn(|_| None),
            routes: route_names
                .iter()
                .map(|name| RouteState::new(name))
                .collect(),
            io: SerialIo::spawn(),
        }
    }

    pub(super) fn route_key(&self, name: &str) -> Option<RouteKey> {
        self.routes
            .iter()
            .position(|route| route.name == name)
            .map(RouteKey)
    }

    pub(super) fn route_name(&self, route: RouteKey) -> &str {
        &self.routes[route.0].name
    }

    pub(super) fn pin_key(&self, route: RouteKey, token: &str) -> Option<PinKey> {
        self.routes
            .get(route.0)?
            .map
            .as_ref()?
            .pins
            .iter()
            .position(|pin| pin.info.token == token)
            .map(|index| PinKey { route, index })
    }

    #[cfg(test)]
    pub(super) fn bank_key(&self, route: RouteKey, token: &str) -> Option<BankKey> {
        self.routes
            .get(route.0)?
            .map
            .as_ref()?
            .banks
            .iter()
            .position(|bank| bank == token)
            .map(|index| BankKey { route, index })
    }

    pub(super) fn pin_info(&self, pin: PinKey) -> Option<&PinInfo> {
        self.routes
            .get(pin.route.0)?
            .map
            .as_ref()?
            .pins
            .get(pin.index)
            .map(|pin| &pin.info)
    }

    pub(super) fn bank_token(&self, bank: BankKey) -> Option<&str> {
        self.routes
            .get(bank.route.0)?
            .map
            .as_ref()?
            .banks
            .get(bank.index)
            .map(String::as_str)
    }

    pub(super) fn pins(&self, route: RouteKey) -> impl Iterator<Item = (PinKey, &PinInfo)> {
        self.routes[route.0]
            .map
            .iter()
            .flat_map(|map| map.pins.iter().enumerate())
            .map(move |(index, pin)| (PinKey { route, index }, &pin.info))
    }

    pub(super) fn banks(&self, route: RouteKey) -> impl Iterator<Item = (BankKey, &str)> {
        self.routes[route.0]
            .map
            .iter()
            .flat_map(|map| map.banks.iter().enumerate())
            .map(move |(index, bank)| (BankKey { route, index }, bank.as_str()))
    }

    pub(super) fn target_pins(&self, route: RouteKey, target: Target) -> Vec<PinKey> {
        let Some(map) = self
            .routes
            .get(route.0)
            .and_then(|route| route.map.as_ref())
        else {
            return Vec::new();
        };
        map.pins
            .iter()
            .enumerate()
            .filter(|(index, pin)| target_contains(route, target, *index, pin.info.bank))
            .map(|(index, _)| PinKey { route, index })
            .collect()
    }

    pub(super) fn pin_state(&self, pin: PinKey) -> Option<PinState> {
        self.routes
            .get(pin.route.0)?
            .map
            .as_ref()?
            .pins
            .get(pin.index)
            .map(|pin| pin.state)
    }

    pub(super) fn change_mode(&mut self, pin: PinKey, mode: Mode) -> Result<Vec<String>, String> {
        let Some(route_pin) = self.route_pin(pin) else {
            return Err("Unknown pin key".into());
        };
        if !mode.supported_by(route_pin.info.capabilities)
            || route_pin.state.target_mode.is_some()
            || route_pin.state.listener.is_pending()
        {
            return Ok(Vec::new());
        }
        self.apply_mode(pin.route, Target::Pin(pin), mode, true)
    }

    pub(super) fn read_pin(&mut self, pin: PinKey) -> Result<Vec<String>, String> {
        let Some(state) = self.pin_state(pin) else {
            return Err("Unknown pin key".into());
        };
        if state.mode.is_none() || state.value_pending {
            return Ok(Vec::new());
        }
        self.read_scope(pin.route, Target::Pin(pin))
    }

    pub(super) fn write_pin(&mut self, pin: PinKey) -> Result<Vec<String>, String> {
        let Some(state) = self.pin_state(pin) else {
            return Err("Unknown pin key".into());
        };
        if state.mode != Some(Mode::Output) || state.value_pending {
            return Ok(Vec::new());
        }
        let level = if state.level == Some(Level::High) {
            Level::Low
        } else {
            Level::High
        };
        self.set_scope_level(pin.route, Target::Pin(pin), level)
    }

    pub(super) fn toggle_listener(&mut self, pin: PinKey) -> Result<Vec<String>, String> {
        let Some(state) = self.pin_state(pin) else {
            return Err("Unknown pin key".into());
        };
        if !state.mode.is_some_and(Mode::is_input) {
            return Ok(Vec::new());
        }
        let enabled = match state.listener {
            ListenerState::Off => true,
            ListenerState::On { .. } => false,
            ListenerState::Enabling { .. } | ListenerState::Disabling { .. } => {
                return Ok(Vec::new());
            }
        };
        self.set_listener_scope(pin.route, Target::Pin(pin), enabled)
    }

    pub(super) fn apply_mode(
        &mut self,
        route: RouteKey,
        target: Target,
        mode: Mode,
        overwrite: bool,
    ) -> Result<Vec<String>, String> {
        if overwrite {
            if self.target_has_pending(route, target) {
                return Ok(Vec::new());
            }
            let mut sent = Vec::with_capacity(2);
            if mode == Mode::Output && self.target_has_listener(route, target) {
                sent.extend(self.set_listener_scope(route, target, false)?);
            }
            self.mark_mode_pending(route, target, mode);
            let request = Request::Direction {
                target,
                direction: mode.direction(),
            };
            sent.push(self.send(route, request)?);
            return Ok(sent);
        }

        let mut sent = Vec::new();
        for pin in self.target_pins(route, target) {
            let Some(route_pin) = self.route_pin(pin) else {
                continue;
            };
            if !route_pin.info.capabilities.available()
                || route_pin.state.mode.is_some()
                || route_pin.state.target_mode.is_some()
            {
                continue;
            }
            let state = &mut self.route_pin_mut(pin).unwrap().state;
            state.target_mode = Some(mode);
            state.level = None;
            let request = Request::Direction {
                target: Target::Pin(pin),
                direction: mode.direction(),
            };
            sent.push(self.send(route, request)?);
        }
        Ok(sent)
    }

    pub(super) fn read_scope(
        &mut self,
        route: RouteKey,
        target: Target,
    ) -> Result<Vec<String>, String> {
        for pin in self.target_pins(route, target) {
            if let Some(route_pin) = self.route_pin_mut(pin)
                && route_pin.state.mode.is_some()
            {
                route_pin.state.value_pending = true;
            }
        }
        let request = Request::Get { target };
        self.send(route, request).map(|line| vec![line])
    }

    pub(super) fn set_listener_scope(
        &mut self,
        route: RouteKey,
        target: Target,
        enabled: bool,
    ) -> Result<Vec<String>, String> {
        let request = Request::Listen {
            target,
            state: enabled.into(),
        };
        let (id, line) = self.send_tracked(route, request)?;
        self.for_target_pins_mut(route, target, |pin| {
            if !pin.state.mode.is_some_and(Mode::is_input) {
                return;
            }
            pin.state.listener = if enabled {
                ListenerState::Enabling { request_id: id }
            } else {
                match pin.state.listener {
                    ListenerState::On { stream_id }
                    | ListenerState::Disabling { stream_id, .. } => ListenerState::Disabling {
                        request_id: id,
                        stream_id,
                    },
                    state => state,
                }
            };
        });
        Ok(vec![line])
    }

    pub(super) fn set_scope_level(
        &mut self,
        route: RouteKey,
        target: Target,
        level: Level,
    ) -> Result<Vec<String>, String> {
        for pin in self.target_pins(route, target) {
            if let Some(route_pin) = self.route_pin_mut(pin)
                && route_pin.state.mode == Some(Mode::Output)
            {
                route_pin.state.value_pending = true;
            }
        }
        let request = Request::Set { target, level };
        self.send(route, request).map(|line| vec![line])
    }

    fn route_pin(&self, pin: PinKey) -> Option<&RoutePin> {
        self.routes
            .get(pin.route.0)?
            .map
            .as_ref()?
            .pins
            .get(pin.index)
    }

    fn route_pin_mut(&mut self, pin: PinKey) -> Option<&mut RoutePin> {
        self.routes
            .get_mut(pin.route.0)?
            .map
            .as_mut()?
            .pins
            .get_mut(pin.index)
    }

    fn target_has_pending(&self, route: RouteKey, target: Target) -> bool {
        self.target_pins(route, target).into_iter().any(|pin| {
            self.pin_state(pin)
                .is_some_and(|state| state.target_mode.is_some() || state.listener.is_pending())
        })
    }

    fn target_has_listener(&self, route: RouteKey, target: Target) -> bool {
        self.target_pins(route, target).into_iter().any(|pin| {
            self.pin_state(pin)
                .is_some_and(|state| state.listener.stream_id().is_some())
        })
    }

    fn mark_mode_pending(&mut self, route: RouteKey, target: Target, mode: Mode) {
        for pin in self.target_pins(route, target) {
            let available = self
                .route_pin(pin)
                .is_some_and(|pin| pin.info.capabilities.available());
            if available && let Some(pin) = self.route_pin_mut(pin) {
                pin.state.target_mode = Some(mode);
                pin.state.level = None;
            }
        }
    }

    fn fail_request_state(&mut self, id: RequestId, route: RouteKey, request: Request) {
        match request {
            Request::Direction { target, .. } | Request::Pullup { target, .. } => {
                for pin in self.target_pins(route, target) {
                    if let Some(pin) = self.route_pin_mut(pin) {
                        pin.state.target_mode = None;
                    }
                }
            }
            Request::Get { target } | Request::Set { target, .. } => {
                for pin in self.target_pins(route, target) {
                    if let Some(pin) = self.route_pin_mut(pin) {
                        pin.state.value_pending = false;
                    }
                }
            }
            Request::Listen { target, state } => {
                self.for_target_pins_mut(route, target, |pin| {
                    pin.state.listener = match (state, pin.state.listener) {
                        (Toggle::On, ListenerState::Enabling { request_id })
                            if request_id == id =>
                        {
                            ListenerState::Off
                        }
                        (
                            Toggle::Off,
                            ListenerState::Disabling {
                                request_id,
                                stream_id,
                            },
                        ) if request_id == id => ListenerState::On { stream_id },
                        (_, listener) => listener,
                    };
                });
            }
            _ => {}
        }
    }

    #[cfg(test)]
    pub(super) fn install_map_for_test(
        &mut self,
        route: RouteKey,
        banks: Vec<String>,
        pins: Vec<(String, usize, u8, PinCapabilities)>,
    ) {
        let pins = pins
            .into_iter()
            .map(|(token, bank, bit, capabilities)| RoutePin {
                info: PinInfo {
                    token,
                    package_pin: None,
                    bank: BankKey { route, index: bank },
                    bit,
                    capabilities,
                },
                state: PinState::UNSET,
            })
            .collect();
        self.routes[route.0].map = Some(RouteMap { banks, pins });
    }

    pub(super) fn available_ports() -> Result<Vec<String>, String> {
        SerialIo::available_ports()
    }

    pub(super) fn connect(&self, name: String) -> Result<(), String> {
        self.io.connect(name)
    }

    pub(super) fn disconnect(&self) -> Result<(), String> {
        self.io.disconnect()
    }

    fn send_tracked(
        &mut self,
        route: RouteKey,
        request: Request,
    ) -> Result<(RequestId, String), String> {
        let (id, frame) = self.prepare(route, request)?;
        let line = String::from_utf8_lossy(frame.as_ref())
            .trim_end_matches(['\r', '\n'])
            .to_owned();
        if let Err(error) = self.io.write(frame) {
            self.cancel(id);
            return Err(error);
        }
        Ok((id, line))
    }

    pub(super) fn send(&mut self, route: RouteKey, request: Request) -> Result<String, String> {
        self.send_tracked(route, request).map(|(_, line)| line)
    }

    pub(super) fn send_raw(&self, line: &str) -> Result<(), String> {
        let mut bytes = line.as_bytes().to_vec();
        bytes.push(b'\n');
        let frame = Frame::try_from(bytes.as_slice())
            .map_err(|_| format!("Raw command exceeds {MAX_PACKET_LEN} bytes including newline"))?;
        self.io.write(frame)
    }

    pub(super) fn poll_listener_updates(&self) {
        self.io.drain_listeners();
    }

    pub(super) fn next_event(&mut self) -> Option<Event> {
        loop {
            match self.io.next_event()? {
                IoEvent::Connected(port) => {
                    self.clear();
                    return Some(Event::Connected(port));
                }
                IoEvent::Disconnected(reason) => {
                    self.clear();
                    return Some(Event::Disconnected(reason));
                }
                IoEvent::Line { line, packet } => {
                    let event = match packet {
                        Ok(packet) => self.received(packet),
                        Err(error) => self.malformed_response(error),
                    };
                    return Some(Event::Received {
                        line: frame_text(&line),
                        event,
                    });
                }
                IoEvent::ListenerValues(values) => {
                    let values = self.accept_listener_values(values);
                    if !values.is_empty() {
                        return Some(Event::ListenerValues(values));
                    }
                }
                IoEvent::Error(error) => return Some(Event::IoError(error)),
            }
        }
    }

    fn malformed_response(&mut self, error: DecodeError) -> Result<DeviceEvent, String> {
        if let Some(id) = error.id {
            self.retire(id);
        }
        Err(format!("Malformed response: {error}"))
    }

    fn prepare(&mut self, route: RouteKey, request: Request) -> Result<(RequestId, Frame), String> {
        if route.0 >= self.routes.len() {
            return Err("Unknown route key".into());
        }
        if matches!(request, Request::Map) && self.routes[route.0].discovery.is_some() {
            return Err(format!(
                "MAP for {} is already in progress",
                self.route_name(route)
            ));
        }

        let id = self.allocate_request_id()?;
        let destination = self.routes[route.0].name.clone();
        let wire_request = request.try_map_target(|target| self.target_token(route, target))?;
        let frame = Frame::try_from(Message {
            route: destination.as_bytes(),
            packet: Packet {
                id,
                body: wire_request,
            },
        })
        .map_err(|error| format!("Could not encode request: {error:?}"))?;

        if matches!(request, Request::Map) {
            self.routes[route.0].discovery = Some(MapBuilder::new(route));
        }
        self.pending[id.slot()] = Some(Pending { route, request });
        Ok((id, frame))
    }

    fn target_token(&self, route: RouteKey, target: Target) -> Result<String, String> {
        match target {
            Target::All => Ok("ALL".into()),
            Target::Pin(pin) if pin.route == route => self
                .pin_info(pin)
                .map(|info| info.token.clone())
                .ok_or_else(|| "Unknown pin key".into()),
            Target::Bank(bank) if bank.route == route => self
                .bank_token(bank)
                .map(str::to_owned)
                .ok_or_else(|| "Unknown bank key".into()),
            Target::Pin(_) | Target::Bank(_) => Err("Target belongs to another route".into()),
        }
    }

    fn allocate_request_id(&mut self) -> Result<RequestId, String> {
        for _ in RequestId::MIN..=RequestId::MAX {
            let id = self.next_id;
            self.next_id = id.next();
            if !self.request_id_in_use(id) {
                return Ok(id);
            }
        }
        Err("All 999 request IDs are still in use".into())
    }

    fn request_id_in_use(&self, id: RequestId) -> bool {
        self.pending[id.slot()].is_some()
            || self.routes.iter().any(|route| {
                route.map.as_ref().is_some_and(|map| {
                    map.pins
                        .iter()
                        .any(|pin| pin.state.listener.stream_id() == Some(id))
                })
            })
    }

    fn received(&mut self, incoming: OwnedResponse) -> Result<DeviceEvent, String> {
        let id = incoming.packet.id;
        let Some(pending) = self.pending[id.slot()] else {
            return Ok(DeviceEvent::Untracked);
        };
        let routing_error = matches!(
            incoming.packet.body,
            WireResponse::Error(
                ProtocolResponseError::NoRoute { .. }
                    | ProtocolResponseError::RouteBusy { .. }
                    | ProtocolResponseError::RouteDown { .. }
            )
        );
        let expected = self.route_name(pending.route);
        if !routing_error && incoming.route != expected {
            return Err(format!(
                "Response {id} came from {}, expected {expected}",
                incoming.route
            ));
        }

        match incoming.packet.body {
            WireResponse::Hello => {
                self.complete(id, false);
                Ok(DeviceEvent::Hello {
                    route: pending.route,
                })
            }
            WireResponse::Status { identity } => {
                self.complete(id, false);
                Ok(DeviceEvent::Status {
                    route: pending.route,
                    identity,
                })
            }
            WireResponse::Version { version } => {
                self.complete(id, false);
                Ok(DeviceEvent::Version {
                    route: pending.route,
                    version,
                })
            }
            WireResponse::Help { command } => {
                self.complete(id, false);
                Ok(DeviceEvent::Help {
                    route: pending.route,
                    command,
                })
            }
            WireResponse::MapBank { bank } => {
                if let Err(error) = self.require_map(pending, id).and_then(|map| map.bank(bank)) {
                    self.retire(id);
                    return Err(error);
                }
                Ok(DeviceEvent::Untracked)
            }
            WireResponse::MapPin {
                target,
                package_pin,
                bank,
                bit,
                capabilities,
            } => {
                if let Err(error) = self
                    .require_map(pending, id)
                    .and_then(|map| map.pin(target, package_pin, bank, bit, capabilities))
                {
                    self.retire(id);
                    return Err(error);
                }
                Ok(DeviceEvent::Untracked)
            }
            WireResponse::Ack => self.ack(id, pending),
            WireResponse::Value { target, level } => {
                let pin = match self.resolve_pin(pending.route, &target) {
                    Ok(pin) => pin,
                    Err(error) => {
                        self.retire(id);
                        return Err(error);
                    }
                };
                if let Some(route_pin) = self.route_pin_mut(pin) {
                    route_pin.state.level = Some(level);
                    route_pin.state.value_pending = false;
                }
                self.complete(id, false);
                Ok(DeviceEvent::PinValue { pin, level })
            }
            WireResponse::State {
                target,
                what,
                value,
            } => {
                let pin = match self.resolve_pin(pending.route, &target) {
                    Ok(pin) => pin,
                    Err(error) => {
                        self.retire(id);
                        return Err(error);
                    }
                };
                self.apply_query_state(pin, what, value);
                self.complete(id, false);
                Ok(DeviceEvent::PinState { pin, what, value })
            }
            WireResponse::Error(error) => {
                let error =
                    match error.try_map(|target| self.resolve_pin(pending.route, &target), Ok) {
                        Ok(error) => error,
                        Err(error) => {
                            self.retire(id);
                            return Err(error);
                        }
                    };
                self.retire(id);
                Ok(DeviceEvent::DeviceError {
                    route: pending.route,
                    source: incoming.route,
                    error,
                })
            }
            WireResponse::Unknown => {
                self.retire(id);
                Ok(DeviceEvent::Unknown {
                    route: pending.route,
                })
            }
            WireResponse::Bye => {
                self.reset_route(pending.route);
                Ok(DeviceEvent::Bye {
                    route: pending.route,
                })
            }
        }
    }

    fn require_map(&mut self, pending: Pending, id: RequestId) -> Result<&mut MapBuilder, String> {
        if pending.request != Request::Map {
            self.retire(id);
            return Err(format!("Unexpected MAP response for request {id}"));
        }
        self.routes[pending.route.0]
            .discovery
            .as_mut()
            .ok_or_else(|| format!("MAP response for {id} has no active discovery"))
    }

    fn resolve_pin(&self, route: RouteKey, token: &str) -> Result<PinKey, String> {
        self.pin_key(route, token).ok_or_else(|| {
            format!(
                "{} response referenced undiscovered pin {token}",
                self.route_name(route)
            )
        })
    }

    fn ack(&mut self, id: RequestId, pending: Pending) -> Result<DeviceEvent, String> {
        let mut follow_up = None;
        match pending.request {
            Request::Map => {
                let Some(builder) = self.routes[pending.route.0].discovery.take() else {
                    self.retire(id);
                    return Err(format!("MAP {id} completed without discovery state"));
                };
                self.routes[pending.route.0].map = Some(builder.finish());
                self.complete(id, true);
                self.sync_listeners();
                return Ok(DeviceEvent::MapReady {
                    route: pending.route,
                });
            }
            Request::Direction { target, direction } => {
                if let Some(mode) = self.pending_mode(pending.route, target) {
                    let request = Request::Pullup {
                        target,
                        state: (mode == Mode::InputPullup).into(),
                    };
                    follow_up = Some(request);
                } else {
                    self.for_target_pins_mut(pending.route, target, |pin| {
                        if pin.info.capabilities.supports_direction(direction) {
                            pin.state.mode = Some(if direction == Direction::Input {
                                Mode::Input
                            } else {
                                Mode::Output
                            });
                            pin.state.level =
                                (direction == Direction::Output).then_some(Level::Low);
                        }
                    });
                }
            }
            Request::Pullup { target, .. } => {
                let mut read = false;
                self.for_target_pins_mut(pending.route, target, |pin| {
                    if let Some(mode) = pin.state.target_mode.take() {
                        pin.state.mode = Some(mode);
                        if mode.is_input() {
                            pin.state.value_pending = true;
                            read = true;
                        } else {
                            pin.state.level = Some(Level::Low);
                            pin.state.value_pending = false;
                        }
                    }
                });
                if read {
                    follow_up = Some(Request::Get { target });
                }
            }
            Request::Set { target, level } => {
                self.for_target_pins_mut(pending.route, target, |pin| {
                    if pin.state.mode == Some(Mode::Output) {
                        pin.state.level = Some(level);
                        pin.state.value_pending = false;
                    }
                });
            }
            Request::Listen { target, state } => {
                self.for_target_pins_mut(pending.route, target, |pin| {
                    pin.state.listener = match (state, pin.state.listener) {
                        (Toggle::On, ListenerState::Enabling { request_id })
                            if request_id == id =>
                        {
                            ListenerState::On { stream_id: id }
                        }
                        (Toggle::Off, ListenerState::Disabling { request_id, .. })
                            if request_id == id =>
                        {
                            ListenerState::Off
                        }
                        (_, listener) => listener,
                    };
                });
                self.sync_listeners();
            }
            _ => {}
        }

        self.complete(id, true);
        let sent = follow_up
            .map(|request| self.send(pending.route, request))
            .transpose()?;
        Ok(DeviceEvent::Ack {
            route: pending.route,
            sent,
        })
    }

    fn pending_mode(&self, route: RouteKey, target: Target) -> Option<Mode> {
        self.target_pins(route, target)
            .into_iter()
            .find_map(|pin| self.pin_state(pin).and_then(|state| state.target_mode))
    }

    fn apply_query_state(&mut self, pin: PinKey, what: Query, value: QueryValue) {
        let Some(route_pin) = self.route_pin_mut(pin) else {
            return;
        };
        match (what, value) {
            (Query::Direction, QueryValue::Direction(Direction::Input)) => {
                if route_pin.state.mode != Some(Mode::InputPullup) {
                    route_pin.state.mode = Some(Mode::Input);
                }
            }
            (Query::Direction, QueryValue::Direction(Direction::Output)) => {
                route_pin.state.mode = Some(Mode::Output);
            }
            (Query::Pullup, QueryValue::Toggle(Toggle::On))
                if route_pin.state.mode.is_some_and(Mode::is_input) =>
            {
                route_pin.state.mode = Some(Mode::InputPullup);
            }
            (Query::Pullup, QueryValue::Toggle(Toggle::Off))
                if route_pin.state.mode.is_some_and(Mode::is_input) =>
            {
                route_pin.state.mode = Some(Mode::Input);
            }
            _ => {}
        }
    }

    fn complete(&mut self, id: RequestId, terminal_ack: bool) {
        let Some(pending) = self.pending[id.slot()] else {
            return;
        };
        let done = match request_lifetime(pending.request) {
            RequestLifetime::OneShot => true,
            RequestLifetime::StreamUntilAck => terminal_ack,
        };
        if done {
            self.pending[id.slot()] = None;
        }
    }

    fn for_target_pins_mut(
        &mut self,
        route: RouteKey,
        target: Target,
        mut apply: impl FnMut(&mut RoutePin),
    ) {
        let Some(map) = self.routes[route.0].map.as_mut() else {
            return;
        };
        for (index, pin) in map.pins.iter_mut().enumerate() {
            if target_contains(route, target, index, pin.info.bank) {
                apply(pin);
            }
        }
    }

    fn listener_is_active(&self, pin: PinKey, id: RequestId) -> bool {
        self.routes
            .get(pin.route.0)
            .and_then(|route| route.map.as_ref())
            .and_then(|map| map.pins.get(pin.index))
            .is_some_and(|pin| pin.state.listener.stream_id() == Some(id))
    }

    fn accept_listener_values(
        &mut self,
        values: Vec<crate::io::ListenerValue>,
    ) -> Vec<ListenerValue> {
        values
            .into_iter()
            .filter_map(|value| {
                let pin = value.key.into();
                if !self.listener_is_active(pin, value.id) {
                    return None;
                }
                self.route_pin_mut(pin)?.state.level = Some(value.level);
                Some(ListenerValue {
                    line: frame_text(&value.line),
                    id: value.id,
                    pin,
                    level: value.level,
                    coalesced: value.coalesced,
                })
            })
            .collect()
    }

    fn retire(&mut self, id: RequestId) {
        let pending = self.pending[id.slot()];
        if let Some(pending) = pending {
            self.fail_request_state(id, pending.route, pending.request);
        }
        if let Some(pending) = pending
            && pending.request == Request::Map
        {
            self.routes[pending.route.0].discovery = None;
        }
        self.pending[id.slot()] = None;
        self.sync_listeners();
    }

    fn cancel(&mut self, id: RequestId) {
        if let Some(pending) = self.pending[id.slot()] {
            self.fail_request_state(id, pending.route, pending.request);
            if pending.request == Request::Map {
                self.routes[pending.route.0].discovery = None;
            }
        }
        self.pending[id.slot()] = None;
    }

    fn reset_route(&mut self, route: RouteKey) {
        if let Some(map) = self.routes[route.0].map.as_mut() {
            for pin in &mut map.pins {
                pin.state = PinState::UNSET;
            }
        }
        self.routes[route.0].discovery = None;
        for pending in &mut self.pending {
            if pending.is_some_and(|pending| pending.route == route) {
                *pending = None;
            }
        }
        self.sync_listeners();
    }

    fn clear(&mut self) {
        self.pending.fill(None);
        for route in &mut self.routes {
            route.map = None;
            route.discovery = None;
        }
        self.sync_listeners();
    }

    fn sync_listeners(&self) {
        let routes = self
            .routes
            .iter()
            .enumerate()
            .map(|(route_index, route)| {
                let pin_count = route.map.as_ref().map_or(0, |map| map.pins.len());
                let pins = route.map.as_ref().map_or_else(Vec::new, |map| {
                    map.pins
                        .iter()
                        .enumerate()
                        .filter_map(|(index, pin)| {
                            pin.state.listener.stream_id().map(|id| ListenerPin {
                                key: PinKey {
                                    route: RouteKey(route_index),
                                    index,
                                }
                                .into(),
                                token: pin.info.token.as_bytes().into(),
                                id,
                            })
                        })
                        .collect()
                });
                ListenerRoute {
                    name: route.name.as_bytes().into(),
                    pin_count,
                    pins,
                }
            })
            .collect();
        self.io.set_listeners(routes);
    }
}

fn target_contains(route: RouteKey, target: Target, pin_index: usize, bank: BankKey) -> bool {
    match target {
        Target::Pin(pin) => pin.route == route && pin.index == pin_index,
        Target::Bank(target) => target == bank,
        Target::All => true,
    }
}

fn frame_text(frame: &Frame) -> String {
    String::from_utf8_lossy(frame.as_ref()).into_owned()
}

fn request_lifetime(request: Request) -> RequestLifetime {
    match request {
        Request::Map
        | Request::Help
        | Request::Get {
            target: Target::Bank(_) | Target::All,
        }
        | Request::Query {
            target: Target::Bank(_) | Target::All,
            ..
        } => RequestLifetime::StreamUntilAck,
        _ => RequestLifetime::OneShot,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_availability_follows_pin_capabilities() {
        assert_eq!(Mode::available_for(PinCapabilities::NONE), []);
        assert_eq!(Mode::available_for(PinCapabilities::INPUT), [Mode::Input]);
        assert_eq!(
            Mode::available_for(PinCapabilities::INPUT_PULLUP),
            [Mode::Input, Mode::InputPullup]
        );
        assert_eq!(Mode::available_for(PinCapabilities::OUTPUT), [Mode::Output]);
        assert_eq!(Mode::available_for(PinCapabilities::GPIO), Mode::ALL);
    }

    #[test]
    fn raw_commands_are_bounded_without_protocol_validation() {
        let connection = DeviceSession::spawn(&["SAM"]);
        assert!(
            connection
                .send_raw("definitely not protocol grammar")
                .is_ok()
        );

        let oversized = "x".repeat(MAX_PACKET_LEN);
        assert_eq!(
            connection.send_raw(&oversized).unwrap_err(),
            format!("Raw command exceeds {MAX_PACKET_LEN} bytes including newline")
        );
    }

    fn setup() -> (DeviceSession, RouteKey, RouteKey, PinKey, PinKey, BankKey) {
        let mut connection = DeviceSession::spawn(&["SAM", "LPC"]);
        let sam = connection.route_key("SAM").unwrap();
        let lpc = connection.route_key("LPC").unwrap();
        connection.routes[sam.0].map = Some(RouteMap {
            banks: vec!["PIOA".into()],
            pins: vec![
                RoutePin {
                    info: PinInfo {
                        token: "PA00".into(),
                        package_pin: Some(102),
                        bank: BankKey {
                            route: sam,
                            index: 0,
                        },
                        bit: 0,
                        capabilities: PinCapabilities::GPIO,
                    },
                    state: PinState::UNSET,
                },
                RoutePin {
                    info: PinInfo {
                        token: "PA01".into(),
                        package_pin: Some(99),
                        bank: BankKey {
                            route: sam,
                            index: 0,
                        },
                        bit: 1,
                        capabilities: PinCapabilities::GPIO,
                    },
                    state: PinState::UNSET,
                },
            ],
        });
        connection.routes[lpc.0].map = Some(RouteMap {
            banks: vec!["PIO2".into()],
            pins: vec![RoutePin {
                info: PinInfo {
                    token: "PIO2_3".into(),
                    package_pin: Some(38),
                    bank: BankKey {
                        route: lpc,
                        index: 0,
                    },
                    bit: 3,
                    capabilities: PinCapabilities::INPUT,
                },
                state: PinState::UNSET,
            }],
        });
        let pa00 = connection.pin_key(sam, "PA00").unwrap();
        let lpc23 = connection.pin_key(lpc, "PIO2_3").unwrap();
        let pioa = connection.bank_key(sam, "PIOA").unwrap();
        (connection, sam, lpc, pa00, lpc23, pioa)
    }

    fn request_id(raw: u16) -> RequestId {
        RequestId::new(raw).unwrap()
    }
    fn line_id(line: &str) -> RequestId {
        request_id(line[..3].parse().unwrap())
    }

    fn incoming(source: &str, id: RequestId, body: WireResponse) -> OwnedResponse {
        OwnedResponse {
            route: source.into(),
            packet: Packet { id, body },
        }
    }

    #[test]
    fn route_request_encoding_uses_discovered_targets_and_global_ids() {
        let (mut connection, sam, lpc, pa00, lpc23, _) = setup();
        let (sam_id, sam_wire) = connection
            .prepare(
                sam,
                Request::Get {
                    target: Target::Pin(pa00),
                },
            )
            .unwrap();
        let (lpc_id, lpc_wire) = connection
            .prepare(
                lpc,
                Request::Get {
                    target: Target::Pin(lpc23),
                },
            )
            .unwrap();
        assert_eq!(sam_id, request_id(1));
        assert_eq!(lpc_id, request_id(2));
        assert_eq!(sam_wire.as_ref(), b"001 SAM GET PA00 OK?\n");
        assert_eq!(lpc_wire.as_ref(), b"002 LPC GET PIO2_3 OK?\n");
    }

    #[test]
    fn route_normal_response_source_is_checked_without_retiring_request() {
        let (mut connection, sam, _, _, _, _) = setup();
        let (id, _) = connection.prepare(sam, Request::Hello).unwrap();
        assert!(
            connection
                .received(incoming("LPC", id, WireResponse::Hello))
                .unwrap_err()
                .contains("expected SAM")
        );
        assert!(connection.pending[id.slot()].is_some());
        assert_eq!(
            connection
                .received(incoming("SAM", id, WireResponse::Hello))
                .unwrap(),
            DeviceEvent::Hello { route: sam }
        );
        assert!(connection.pending[id.slot()].is_none());
    }

    #[test]
    fn route_intermediate_error_can_retire_downstream_request() {
        let (mut connection, _, lpc, _, _, _) = setup();
        let (id, _) = connection.prepare(lpc, Request::Hello).unwrap();
        assert_eq!(
            connection
                .received(incoming(
                    "SAM",
                    id,
                    WireResponse::Error(ProtocolResponseError::RouteDown {
                        next_hop: "LPC".into(),
                    }),
                ))
                .unwrap(),
            DeviceEvent::DeviceError {
                route: lpc,
                source: "SAM".into(),
                error: ProtocolResponseError::RouteDown {
                    next_hop: "LPC".into(),
                },
            }
        );
        assert!(connection.pending[id.slot()].is_none());
    }

    #[test]
    fn map_stream_builds_dynamic_route_state_only_on_ack() {
        let (mut connection, sam, _, _, _, _) = setup();
        connection.routes[sam.0].map = None;
        let (id, wire) = connection.prepare(sam, Request::Map).unwrap();
        assert_eq!(wire.as_ref(), b"001 SAM MAP\n");
        assert_eq!(
            request_lifetime(Request::Map),
            RequestLifetime::StreamUntilAck
        );

        for body in [
            WireResponse::MapBank {
                bank: "GPIO0".into(),
            },
            WireResponse::MapBank {
                bank: "GPIO1".into(),
            },
            WireResponse::MapPin {
                target: "P0_7".into(),
                package_pin: None,
                bank: "GPIO0".into(),
                bit: 7,
                capabilities: PinCapabilities::INPUT_PULLUP,
            },
            WireResponse::MapPin {
                target: "LED_A".into(),
                package_pin: Some(48),
                bank: "GPIO1".into(),
                bit: 3,
                capabilities: PinCapabilities::GPIO,
            },
        ] {
            assert_eq!(
                connection.received(incoming("SAM", id, body)).unwrap(),
                DeviceEvent::Untracked
            );
            assert!(connection.routes[sam.0].map.is_none());
            assert!(connection.pending[id.slot()].is_some());
        }

        assert_eq!(
            connection
                .received(incoming("SAM", id, WireResponse::Ack))
                .unwrap(),
            DeviceEvent::MapReady { route: sam }
        );
        assert!(connection.pending[id.slot()].is_none());
        assert_eq!(connection.banks(sam).count(), 2);
        assert_eq!(connection.pins(sam).count(), 2);
        let led = connection.pin_key(sam, "LED_A").unwrap();
        assert_eq!(connection.pin_info(led).unwrap().package_pin, Some(48));
        assert!(connection.pin_info(led).unwrap().capabilities.output());
    }

    #[test]
    fn help_stream_stays_pending_until_ack_and_version_is_one_shot() {
        let (mut connection, sam, _, _, _, _) = setup();

        let (version_id, version_wire) = connection.prepare(sam, Request::Version).unwrap();
        assert_eq!(version_wire.as_ref(), b"001 SAM VER\n");
        assert_eq!(request_lifetime(Request::Version), RequestLifetime::OneShot);
        assert_eq!(
            connection
                .received(incoming(
                    "SAM",
                    version_id,
                    WireResponse::Version { version: 1 },
                ))
                .unwrap(),
            DeviceEvent::Version {
                route: sam,
                version: 1,
            }
        );
        assert!(connection.pending[version_id.slot()].is_none());

        let (help_id, help_wire) = connection.prepare(sam, Request::Help).unwrap();
        assert_eq!(help_wire.as_ref(), b"002 SAM HLP\n");
        assert_eq!(
            request_lifetime(Request::Help),
            RequestLifetime::StreamUntilAck
        );
        for command in [Command::Hello, Command::Help] {
            assert_eq!(
                connection
                    .received(incoming("SAM", help_id, WireResponse::Help { command }))
                    .unwrap(),
                DeviceEvent::Help {
                    route: sam,
                    command,
                }
            );
            assert!(connection.pending[help_id.slot()].is_some());
        }
        assert_eq!(
            connection
                .received(incoming("SAM", help_id, WireResponse::Ack))
                .unwrap(),
            DeviceEvent::Ack {
                route: sam,
                sent: None,
            }
        );
        assert!(connection.pending[help_id.slot()].is_none());
    }

    #[test]
    fn malformed_correlated_response_retires_partial_map() {
        let (mut connection, sam, _, _, _, _) = setup();
        connection.routes[sam.0].map = None;
        let (id, _) = connection.prepare(sam, Request::Map).unwrap();
        connection.routes[sam.0]
            .discovery
            .as_mut()
            .unwrap()
            .bank("PIOA".into())
            .unwrap();

        let event = connection.malformed_response(DecodeError {
            id: Some(id),
            kind: da_vinci_protocol::DecodeErrorKind::Malformed,
        });
        assert!(event.unwrap_err().contains("Malformed response"));
        assert!(connection.pending[id.slot()].is_none());
        assert!(connection.routes[sam.0].discovery.is_none());
    }

    #[test]
    fn map_validation_failure_retires_request_and_discards_partial_map() {
        let (mut connection, sam, _, _, _, _) = setup();
        connection.routes[sam.0].map = None;
        let (id, _) = connection.prepare(sam, Request::Map).unwrap();
        connection
            .received(incoming(
                "SAM",
                id,
                WireResponse::MapBank {
                    bank: "PIOA".into(),
                },
            ))
            .unwrap();

        let error = connection
            .received(incoming(
                "SAM",
                id,
                WireResponse::MapPin {
                    target: "PA00".into(),
                    package_pin: Some(102),
                    bank: "MISSING".into(),
                    bit: 0,
                    capabilities: PinCapabilities::GPIO,
                },
            ))
            .unwrap_err();

        assert!(error.contains("unknown bank MISSING"));
        assert!(connection.pending[id.slot()].is_none());
        assert!(connection.routes[sam.0].discovery.is_none());
        assert!(connection.routes[sam.0].map.is_none());
    }

    #[test]
    fn route_target_keys_cannot_cross_routes() {
        let (mut connection, sam, lpc, _, lpc23, _) = setup();
        let error = connection
            .prepare(
                sam,
                Request::Get {
                    target: Target::Pin(lpc23),
                },
            )
            .unwrap_err();
        assert_eq!(error, "Target belongs to another route");
        assert!(connection.prepare(lpc, Request::Map).is_ok());
        assert!(connection.prepare(lpc, Request::Map).is_err());
    }

    #[test]
    fn map_discovery_only_blocks_a_second_map() {
        let (mut connection, sam, _, _, _, _) = setup();
        connection.routes[sam.0].map = None;
        let (map_id, _) = connection.prepare(sam, Request::Map).unwrap();

        let (hello_id, hello) = connection.prepare(sam, Request::Hello).unwrap();
        let (status_id, status) = connection.prepare(sam, Request::Status).unwrap();
        assert_eq!(hello.as_ref(), b"002 SAM HAI\n");
        assert_eq!(status.as_ref(), b"003 SAM HRU\n");
        assert_ne!(map_id, hello_id);
        assert_ne!(map_id, status_id);
        assert_eq!(
            connection.prepare(sam, Request::Map).unwrap_err(),
            "MAP for SAM is already in progress"
        );
        assert!(connection.pending[map_id.slot()].is_some());
        assert!(connection.routes[sam.0].discovery.is_some());
    }

    #[test]
    fn map_refresh_keeps_installed_map_until_ack() {
        let (mut connection, sam, _, pa00, _, _) = setup();
        let (id, _) = connection.prepare(sam, Request::Map).unwrap();

        connection
            .received(incoming(
                "SAM",
                id,
                WireResponse::MapBank {
                    bank: "GPIOX".into(),
                },
            ))
            .unwrap();
        connection
            .received(incoming(
                "SAM",
                id,
                WireResponse::MapPin {
                    target: "X0".into(),
                    package_pin: Some(7),
                    bank: "GPIOX".into(),
                    bit: 0,
                    capabilities: PinCapabilities::INPUT,
                },
            ))
            .unwrap();

        assert_eq!(connection.pin_key(sam, "PA00"), Some(pa00));
        assert_eq!(connection.pin_key(sam, "X0"), None);

        assert_eq!(
            connection
                .received(incoming("SAM", id, WireResponse::Ack))
                .unwrap(),
            DeviceEvent::MapReady { route: sam }
        );
        assert_eq!(connection.pin_key(sam, "PA00"), None);
        assert!(connection.pin_key(sam, "X0").is_some());
    }

    #[test]
    fn failed_map_refresh_preserves_installed_map() {
        let (mut connection, sam, _, pa00, _, _) = setup();
        let (id, _) = connection.prepare(sam, Request::Map).unwrap();
        connection
            .received(incoming(
                "SAM",
                id,
                WireResponse::MapBank {
                    bank: "GPIOX".into(),
                },
            ))
            .unwrap();

        let error = connection
            .received(incoming(
                "SAM",
                id,
                WireResponse::MapPin {
                    target: "X0".into(),
                    package_pin: None,
                    bank: "MISSING".into(),
                    bit: 0,
                    capabilities: PinCapabilities::INPUT,
                },
            ))
            .unwrap_err();

        assert!(error.contains("unknown bank MISSING"));
        assert_eq!(connection.pin_key(sam, "PA00"), Some(pa00));
        assert!(connection.routes[sam.0].discovery.is_none());
        assert!(connection.pending[id.slot()].is_none());
    }

    #[test]
    fn coalesced_listener_values_update_session_level_without_completing_read() {
        let (mut connection, sam, _, pa00, _, _) = setup();
        let listener_id = request_id(8);
        let stale_id = request_id(9);
        let pin = connection.route_pin_mut(pa00).unwrap();
        pin.state.listener = ListenerState::On {
            stream_id: listener_id,
        };
        pin.state.value_pending = true;

        let values = connection.accept_listener_values(vec![
            crate::io::ListenerValue {
                line: Frame::try_from(b"008 SAM HYG PA00 HIGH <3".as_slice()).unwrap(),
                id: listener_id,
                key: ListenerKey {
                    route: sam.0,
                    pin: pa00.index,
                },
                level: Level::High,
                coalesced: 3,
            },
            crate::io::ListenerValue {
                line: Frame::try_from(b"009 SAM HYG PA00 LOW <3".as_slice()).unwrap(),
                id: stale_id,
                key: ListenerKey {
                    route: sam.0,
                    pin: pa00.index,
                },
                level: Level::Low,
                coalesced: 0,
            },
        ]);

        assert_eq!(values.len(), 1);
        assert_eq!(values[0].level, Level::High);
        assert_eq!(values[0].coalesced, 3);
        assert_eq!(connection.pin_state(pa00).unwrap().level, Some(Level::High));
        assert!(connection.pin_state(pa00).unwrap().value_pending);

        let values = connection.accept_listener_values(vec![crate::io::ListenerValue {
            line: Frame::try_from(b"008 SAM HYG PA00 LOW <3".as_slice()).unwrap(),
            id: listener_id,
            key: ListenerKey {
                route: sam.0,
                pin: pa00.index,
            },
            level: Level::Low,
            coalesced: 0,
        }]);
        assert_eq!(values.len(), 1);
        assert_eq!(connection.pin_state(pa00).unwrap().level, Some(Level::Low));
        assert!(connection.pin_state(pa00).unwrap().value_pending);
    }

    #[test]
    fn mode_change_followups_update_session_state_once() {
        let (mut connection, _, _, pa00, _, _) = setup();

        let sent = connection.change_mode(pa00, Mode::InputPullup).unwrap();
        assert_eq!(sent.len(), 1);
        assert_eq!(
            connection.pin_state(pa00).unwrap().target_mode,
            Some(Mode::InputPullup)
        );

        let direction_id = line_id(&sent[0]);
        let DeviceEvent::Ack {
            sent: Some(pullup), ..
        } = connection
            .received(incoming("SAM", direction_id, WireResponse::Ack))
            .unwrap()
        else {
            panic!("direction ACK should schedule pull-up configuration");
        };
        assert!(connection.pending[direction_id.slot()].is_none());

        let pullup_id = line_id(&pullup);
        let DeviceEvent::Ack {
            sent: Some(read), ..
        } = connection
            .received(incoming("SAM", pullup_id, WireResponse::Ack))
            .unwrap()
        else {
            panic!("pull-up ACK should schedule an input read");
        };
        assert!(connection.pending[pullup_id.slot()].is_none());
        assert_eq!(
            connection.pin_state(pa00).unwrap(),
            PinState {
                mode: Some(Mode::InputPullup),
                target_mode: None,
                level: None,
                listener: ListenerState::Off,
                value_pending: true,
            }
        );

        let read_id = line_id(&read);
        connection
            .received(incoming(
                "SAM",
                read_id,
                WireResponse::Value {
                    target: "PA00".into(),
                    level: Level::High,
                },
            ))
            .unwrap();
        assert!(connection.pending[read_id.slot()].is_none());
        assert_eq!(connection.pin_state(pa00).unwrap().level, Some(Level::High));
        assert!(!connection.pin_state(pa00).unwrap().value_pending);
    }

    #[test]
    fn output_mode_stops_listener_and_initializes_low() {
        let (mut connection, _, _, pa00, _, _) = setup();
        let stream_id = request_id(8);
        connection.route_pin_mut(pa00).unwrap().state = PinState {
            mode: Some(Mode::Input),
            target_mode: None,
            level: Some(Level::High),
            listener: ListenerState::On { stream_id },
            value_pending: false,
        };

        let sent = connection.change_mode(pa00, Mode::Output).unwrap();
        assert_eq!(sent.len(), 2);
        assert!(sent[0].contains("LSN PA00 OFF"));
        assert!(sent[1].contains("DIR PA00 OUT"));
        assert_eq!(
            connection.pin_state(pa00).unwrap().listener,
            ListenerState::Disabling {
                request_id: line_id(&sent[0]),
                stream_id,
            }
        );
        assert_eq!(
            connection.pin_state(pa00).unwrap().target_mode,
            Some(Mode::Output)
        );

        connection
            .received(incoming("SAM", line_id(&sent[0]), WireResponse::Ack))
            .unwrap();
        let DeviceEvent::Ack {
            sent: Some(pullup), ..
        } = connection
            .received(incoming("SAM", line_id(&sent[1]), WireResponse::Ack))
            .unwrap()
        else {
            panic!("direction ACK should clear pull-up state");
        };
        connection
            .received(incoming("SAM", line_id(&pullup), WireResponse::Ack))
            .unwrap();

        assert_eq!(
            connection.pin_state(pa00).unwrap(),
            PinState {
                mode: Some(Mode::Output),
                target_mode: None,
                level: Some(Level::Low),
                listener: ListenerState::Off,
                value_pending: false,
            }
        );
    }

    #[test]
    fn failed_followup_send_does_not_leak_completed_request() {
        let (mut connection, sam, _, pa00, _, _) = setup();
        connection.route_pin_mut(pa00).unwrap().state.target_mode = Some(Mode::InputPullup);
        let (direction_id, _) = connection
            .prepare(
                sam,
                Request::Direction {
                    target: Target::Pin(pa00),
                    direction: Direction::Input,
                },
            )
            .unwrap();
        connection.io.stop_for_test();

        let error = connection
            .received(incoming("SAM", direction_id, WireResponse::Ack))
            .unwrap_err();

        assert_eq!(error, "Serial worker stopped");
        assert!(connection.pending[direction_id.slot()].is_none());
        assert!(connection.pending.iter().all(Option::is_none));
        assert_eq!(connection.pin_state(pa00).unwrap().target_mode, None);
    }

    #[test]
    fn routing_error_retires_staged_state_for_destination_route() {
        let (mut connection, _, _, _, lpc23, _) = setup();
        let sent = connection.change_mode(lpc23, Mode::Input).unwrap();
        let id = line_id(&sent[0]);

        connection
            .received(incoming(
                "SAM",
                id,
                WireResponse::Error(ProtocolResponseError::RouteDown {
                    next_hop: "LPC".into(),
                }),
            ))
            .unwrap();

        assert!(connection.pending[id.slot()].is_none());
        assert_eq!(connection.pin_state(lpc23).unwrap().target_mode, None);
        assert_eq!(connection.pin_state(lpc23).unwrap().mode, None);
    }

    #[test]
    fn resetting_one_route_preserves_unrelated_route_state_and_requests() {
        let (mut connection, sam, lpc, pa00, lpc23, _) = setup();
        let sam_stream = request_id(8);
        connection.route_pin_mut(pa00).unwrap().state.listener = ListenerState::On {
            stream_id: sam_stream,
        };
        connection.route_pin_mut(lpc23).unwrap().state.mode = Some(Mode::Input);
        let (lpc_request, _) = connection.prepare(lpc, Request::Hello).unwrap();
        let (sam_reset, _) = connection.prepare(sam, Request::Bye).unwrap();

        assert_eq!(
            connection
                .received(incoming("SAM", sam_reset, WireResponse::Bye))
                .unwrap(),
            DeviceEvent::Bye { route: sam }
        );

        assert!(connection.pending[sam_reset.slot()].is_none());
        assert!(connection.pending[lpc_request.slot()].is_some());
        assert!(!connection.request_id_in_use(sam_stream));
        assert_eq!(connection.pin_state(pa00).unwrap(), PinState::UNSET);
        assert_eq!(connection.pin_state(lpc23).unwrap().mode, Some(Mode::Input));
    }

    #[test]
    fn listener_stream_id_outlives_acked_request_and_grouped_streams_end_at_ack() {
        let (mut connection, sam, _, pa00, _, pioa) = setup();
        connection.route_pin_mut(pa00).unwrap().state.mode = Some(Mode::Input);

        let listen = connection
            .set_listener_scope(sam, Target::Pin(pa00), true)
            .unwrap();
        let listen_id = line_id(&listen[0]);
        assert_eq!(
            connection.pin_state(pa00).unwrap().listener,
            ListenerState::Enabling {
                request_id: listen_id,
            }
        );
        connection
            .received(incoming("SAM", listen_id, WireResponse::Ack))
            .unwrap();
        assert!(connection.pending[listen_id.slot()].is_none());
        assert_eq!(
            connection.pin_state(pa00).unwrap().listener,
            ListenerState::On {
                stream_id: listen_id,
            }
        );

        let (get_id, _) = connection
            .prepare(
                sam,
                Request::Get {
                    target: Target::Bank(pioa),
                },
            )
            .unwrap();
        connection
            .received(incoming(
                "SAM",
                get_id,
                WireResponse::Value {
                    target: "PA00".into(),
                    level: Level::Low,
                },
            ))
            .unwrap();
        assert!(connection.pending[get_id.slot()].is_some());
        connection
            .received(incoming("SAM", get_id, WireResponse::Ack))
            .unwrap();
        assert!(connection.pending[get_id.slot()].is_none());
    }

    #[test]
    fn request_ids_wrap_and_skip_active_listener_stream() {
        let (mut connection, sam, _, pa00, _, _) = setup();
        connection.route_pin_mut(pa00).unwrap().state.mode = Some(Mode::Input);
        let listener = connection
            .set_listener_scope(sam, Target::Pin(pa00), true)
            .unwrap();
        let listener = line_id(&listener[0]);
        connection
            .received(incoming("SAM", listener, WireResponse::Ack))
            .unwrap();

        for _ in 2..=RequestId::MAX {
            let (id, _) = connection.prepare(sam, Request::Hello).unwrap();
            connection
                .received(incoming("SAM", id, WireResponse::Hello))
                .unwrap();
        }
        let (id, _) = connection.prepare(sam, Request::Hello).unwrap();
        assert_eq!(id, request_id(2));
        assert!(connection.pending[listener.slot()].is_none());
        assert_eq!(
            connection.pin_state(pa00).unwrap().listener,
            ListenerState::On {
                stream_id: listener,
            }
        );
    }

    #[test]
    fn failed_listener_transitions_restore_the_previous_semantic_state() {
        let (mut connection, sam, _, pa00, _, _) = setup();
        connection.route_pin_mut(pa00).unwrap().state.mode = Some(Mode::Input);

        let sent = connection
            .set_listener_scope(sam, Target::Pin(pa00), true)
            .unwrap();
        let on_id = line_id(&sent[0]);
        connection
            .received(incoming(
                "SAM",
                on_id,
                WireResponse::Error(ProtocolResponseError::BadPacket),
            ))
            .unwrap();
        assert_eq!(
            connection.pin_state(pa00).unwrap().listener,
            ListenerState::Off
        );

        let stream_id = request_id(8);
        connection.route_pin_mut(pa00).unwrap().state.listener = ListenerState::On { stream_id };
        let sent = connection
            .set_listener_scope(sam, Target::Pin(pa00), false)
            .unwrap();
        let off_id = line_id(&sent[0]);
        assert_eq!(
            connection.pin_state(pa00).unwrap().listener,
            ListenerState::Disabling {
                request_id: off_id,
                stream_id,
            }
        );
        connection
            .received(incoming(
                "SAM",
                off_id,
                WireResponse::Error(ProtocolResponseError::BadPacket),
            ))
            .unwrap();
        assert_eq!(
            connection.pin_state(pa00).unwrap().listener,
            ListenerState::On { stream_id }
        );
    }

    #[test]
    fn grouped_listener_stop_preserves_each_stream_until_ack() {
        let (mut connection, sam, _, pa00, _, pioa) = setup();
        let pa01 = connection.pin_key(sam, "PA01").unwrap();
        let first = request_id(8);
        let second = request_id(9);
        connection.route_pin_mut(pa00).unwrap().state = PinState {
            mode: Some(Mode::Input),
            listener: ListenerState::On { stream_id: first },
            ..PinState::UNSET
        };
        connection.route_pin_mut(pa01).unwrap().state = PinState {
            mode: Some(Mode::Input),
            listener: ListenerState::On { stream_id: second },
            ..PinState::UNSET
        };

        let sent = connection
            .set_listener_scope(sam, Target::Bank(pioa), false)
            .unwrap();
        let off_id = line_id(&sent[0]);
        assert_eq!(
            connection.pin_state(pa00).unwrap().listener,
            ListenerState::Disabling {
                request_id: off_id,
                stream_id: first,
            }
        );
        assert_eq!(
            connection.pin_state(pa01).unwrap().listener,
            ListenerState::Disabling {
                request_id: off_id,
                stream_id: second,
            }
        );
        assert!(connection.listener_is_active(pa00, first));
        assert!(connection.listener_is_active(pa01, second));

        connection
            .received(incoming("SAM", off_id, WireResponse::Ack))
            .unwrap();
        assert_eq!(
            connection.pin_state(pa00).unwrap().listener,
            ListenerState::Off
        );
        assert_eq!(
            connection.pin_state(pa01).unwrap().listener,
            ListenerState::Off
        );
    }
}
