use std::{collections::VecDeque, fmt, path::Path, time::Duration};

use da_vinci_protocol::Level;
use iced::{
    Element, Subscription, Task,
    keyboard::{Event as KeyboardEvent, Key, key::Named},
    widget::{pane_grid, text_editor},
};

use crate::{
    serial_log::SerialLog,
    session::{
        BankKey, DeviceEvent, DeviceSession, Event as ConnectionEvent, Mode, PinKey,
        Request as RoutedRequest, ResponseError, RouteKey, Target as RoutedTarget,
    },
    view,
};

const MAX_IO_EVENTS_PER_TICK: usize = 256;
const MAX_COMMAND_HISTORY: usize = 200;
const ROUTES: [&str; 2] = ["SAM", "LPC"];

type Request = RoutedRequest;

pub(super) const MODES: [Mode; 3] = [Mode::Input, Mode::InputPullup, Mode::Output];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ScopeChoice {
    target: RoutedTarget,
    label: String,
}

impl ScopeChoice {
    fn all() -> Self {
        Self {
            target: RoutedTarget::All,
            label: "ALL".into(),
        }
    }
}

impl fmt::Display for ScopeChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PortChoice {
    path: String,
    label: String,
}

impl PortChoice {
    fn new(path: String) -> Self {
        let label = Path::new(&path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(&path)
            .to_owned();
        Self { path, label }
    }
}

impl fmt::Display for PortChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RouteChoice {
    key: RouteKey,
    label: String,
}

impl fmt::Display for RouteChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label)
    }
}

#[derive(Debug)]
pub(super) struct BankGroup {
    pub(super) label: String,
    pub(super) banks: Vec<BankKey>,
}

#[derive(Clone, Copy)]
struct BankGroupSpec {
    label: &'static str,
    banks: &'static [&'static str],
}

#[derive(Clone, Copy)]
struct RouteLayout {
    route: RouteKey,
    groups: &'static [BankGroupSpec],
}

const SAM_BANK_GROUPS: &[BankGroupSpec] = &[
    BankGroupSpec {
        label: "PIOA",
        banks: &["PIOA"],
    },
    BankGroupSpec {
        label: "PIOB + PIOE",
        banks: &["PIOB", "PIOE"],
    },
    BankGroupSpec {
        label: "PIOC",
        banks: &["PIOC"],
    },
    BankGroupSpec {
        label: "PIOD",
        banks: &["PIOD"],
    },
];

#[derive(Clone, Copy, Debug)]
pub(super) enum PaneKind {
    Pins,
    Log,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum HistoryDirection {
    Previous,
    Next,
}

#[derive(Clone, Debug)]
pub(super) enum Message {
    Tick,
    PortsLoaded(Result<Vec<String>, String>),
    RefreshPorts,
    PortSelected(PortChoice),
    Connect,
    Disconnect,
    RouteSelected(RouteChoice),
    PreviousGroup,
    NextGroup,
    GroupSelected(usize),
    ModeSelected(PinKey, Mode),
    Read(PinKey),
    Write(PinKey),
    Listen(PinKey),
    BulkScopeSelected(ScopeChoice),
    BulkModeSelected(Mode),
    OverwriteChanged(bool),
    ApplyBulkMode,
    BulkRead,
    BulkListen(bool),
    BulkSet(Level),
    BulkSetConfirm,
    BulkSetCancel,
    PinMap,
    Handshake,
    Status,
    Version,
    Help,
    Reboot,
    RebootConfirm,
    RebootCancel,
    PaneResized(pane_grid::ResizeEvent),
    ClearLog,
    ShowTimestamps(bool),
    Autoscroll(bool),
    LogAction(text_editor::Action),
    RawChanged(String),
    RawSend,
    HistoryKey(HistoryDirection),
    HistoryKeyFocus {
        direction: HistoryDirection,
        focused: bool,
    },
}

pub(super) struct App {
    pub(super) routes: Vec<RouteChoice>,
    pub(super) selected_route: RouteChoice,
    route_layout: Option<RouteLayout>,
    pub(super) bank_groups: Vec<BankGroup>,
    pub(super) bank_group: usize,
    pub(super) bulk_scope: ScopeChoice,
    pub(super) bulk_scopes: Vec<ScopeChoice>,
    pub(super) bulk_mode: Mode,
    pub(super) overwrite: bool,
    pub(super) confirm_set: Option<(ScopeChoice, Level)>,
    pub(super) panes: pane_grid::State<PaneKind>,
    pub(super) ports: Vec<PortChoice>,
    pub(super) selected_port: Option<PortChoice>,
    pub(super) connected_port: Option<String>,
    pub(super) session: DeviceSession,
    pub(super) log: SerialLog,
    pub(super) autoscroll: bool,
    pub(super) log_scroll: iced::widget::Id,
    pub(super) raw_input_id: iced::widget::Id,
    pub(super) raw_input: String,
    command_history: VecDeque<String>,
    history_index: Option<usize>,
    pub(super) device_status: String,
    pub(super) error: Option<String>,
    pub(super) confirm_reboot: bool,
}

impl App {
    pub(super) fn new() -> (Self, Task<Message>) {
        (Self::with_routes(&ROUTES), load_ports())
    }

    fn with_routes(route_names: &[&str]) -> Self {
        let (mut panes, pins_pane) = pane_grid::State::new(PaneKind::Pins);
        let (_, split) = panes
            .split(pane_grid::Axis::Vertical, pins_pane, PaneKind::Log)
            .expect("initial GPIO/log split must succeed");
        panes.resize(split, 0.76);

        let session = DeviceSession::spawn(route_names);
        let routes: Vec<_> = route_names
            .iter()
            .map(|name| RouteChoice {
                key: session
                    .route_key(name)
                    .expect("configured route must exist"),
                label: (*name).to_owned(),
            })
            .collect();
        let selected_route = routes
            .first()
            .cloned()
            .expect("at least one route is configured");
        let route_layout = session.route_key("SAM").map(|route| RouteLayout {
            route,
            groups: SAM_BANK_GROUPS,
        });

        Self {
            routes,
            selected_route,
            route_layout,
            bank_groups: Vec::new(),
            bank_group: 0,
            bulk_scope: ScopeChoice::all(),
            bulk_scopes: vec![ScopeChoice::all()],
            bulk_mode: Mode::Input,
            overwrite: false,
            confirm_set: None,
            panes,
            ports: Vec::new(),
            selected_port: None,
            connected_port: None,
            session,
            log: SerialLog::new(),
            autoscroll: true,
            log_scroll: iced::widget::Id::unique(),
            raw_input_id: iced::widget::Id::unique(),
            raw_input: String::new(),
            command_history: VecDeque::new(),
            history_index: None,
            device_status: "Disconnected".into(),
            error: None,
            confirm_reboot: false,
        }
    }

    pub(super) fn view(&self) -> Element<'_, Message> {
        view::view(self)
    }

    pub(super) fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            iced::time::every(Duration::from_millis(40)).map(|_| Message::Tick),
            iced::event::listen_with(|event, _, _| match event {
                iced::Event::Keyboard(KeyboardEvent::KeyPressed {
                    key: Key::Named(Named::ArrowUp),
                    ..
                }) => Some(Message::HistoryKey(HistoryDirection::Previous)),
                iced::Event::Keyboard(KeyboardEvent::KeyPressed {
                    key: Key::Named(Named::ArrowDown),
                    ..
                }) => Some(Message::HistoryKey(HistoryDirection::Next)),
                _ => None,
            }),
        ])
    }

    pub(super) fn update(&mut self, message: Message) -> Task<Message> {
        let task = self.handle_message(message);
        if self.log.flush() {
            Task::batch([task, self.snap_log()])
        } else {
            task
        }
    }

    fn handle_message(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => self.drain_io(),
            Message::PortsLoaded(result) => match result {
                Ok(ports) => {
                    self.ports = ports.into_iter().map(PortChoice::new).collect();
                    if self
                        .selected_port
                        .as_ref()
                        .is_none_or(|selected| !self.ports.contains(selected))
                    {
                        self.selected_port = self.ports.first().cloned();
                    }
                }
                Err(error) => self.error = Some(error),
            },
            Message::RefreshPorts => return load_ports(),
            Message::PortSelected(port) => self.selected_port = Some(port),
            Message::Connect => {
                if let Some(port) = &self.selected_port {
                    self.error = self.session.connect(port.path.clone()).err();
                }
            }
            Message::Disconnect => self.error = self.session.disconnect().err(),
            Message::RouteSelected(route) => {
                self.selected_route = route;
                self.sync_route_ui();
            }
            Message::PreviousGroup => {
                self.bank_group = self.bank_group.saturating_sub(1);
            }
            Message::NextGroup => {
                if self.bank_group + 1 < self.bank_groups.len() {
                    self.bank_group += 1;
                }
            }
            Message::GroupSelected(index) => {
                if index < self.bank_groups.len() {
                    self.bank_group = index;
                }
            }
            Message::ModeSelected(pin, mode) => self.change_mode(pin, mode),
            Message::Read(pin) => self.read_pin(pin),
            Message::Write(pin) => self.write_pin(pin),
            Message::Listen(pin) => self.toggle_listener(pin),
            Message::BulkScopeSelected(scope) => {
                self.bulk_scope = scope;
                self.confirm_set = None;
                self.normalize_bulk_mode();
            }
            Message::BulkModeSelected(mode) => self.bulk_mode = mode,
            Message::OverwriteChanged(overwrite) => self.overwrite = overwrite,
            Message::ApplyBulkMode => self.apply_bulk_mode(),
            Message::BulkRead => self.read_scope(self.bulk_scope.target),
            Message::BulkListen(enabled) => {
                self.set_listener_scope(self.bulk_scope.target, enabled)
            }
            Message::BulkSet(level) => self.confirm_set = Some((self.bulk_scope.clone(), level)),
            Message::BulkSetConfirm => {
                if let Some((target, level)) = self.confirm_set.take() {
                    self.set_scope_level(target.target, level);
                }
            }
            Message::BulkSetCancel => self.confirm_set = None,
            Message::PinMap => self.send_request(Request::Map),
            Message::Handshake => self.send_request(Request::Hello),
            Message::Status => self.send_request(Request::Status),
            Message::Version => self.send_request(Request::Version),
            Message::Help => self.send_request(Request::Help),
            Message::Reboot => self.confirm_reboot = true,
            Message::RebootConfirm => {
                self.confirm_reboot = false;
                self.send_request(Request::Bye);
            }
            Message::RebootCancel => self.confirm_reboot = false,
            Message::PaneResized(event) => self.panes.resize(event.split, event.ratio),
            Message::ClearLog => self.log.clear(),
            Message::ShowTimestamps(enabled) => self.log.set_show_timestamps(enabled),
            Message::Autoscroll(enabled) => {
                self.autoscroll = enabled;
                if enabled {
                    return self.snap_log();
                }
            }
            Message::LogAction(action) => {
                if !action.is_edit() {
                    self.log.perform(action);
                }
            }
            Message::RawChanged(value) => {
                self.raw_input = value;
                self.history_index = None;
            }
            Message::RawSend => self.send_raw(),
            Message::HistoryKey(direction) => {
                return iced::widget::operation::is_focused(self.raw_input_id.clone())
                    .map(move |focused| Message::HistoryKeyFocus { direction, focused });
            }
            Message::HistoryKeyFocus { direction, focused } => {
                if focused {
                    match direction {
                        HistoryDirection::Previous => self.history_previous(),
                        HistoryDirection::Next => self.history_next(),
                    }
                }
            }
        }
        Task::none()
    }

    fn change_mode(&mut self, pin: PinKey, mode: Mode) {
        if !self.require_connection() {
            return;
        }
        let result = self.session.change_mode(pin, mode);
        self.record_session_action(result);
    }

    fn read_pin(&mut self, pin: PinKey) {
        if !self.require_connection() {
            return;
        }
        let result = self.session.read_pin(pin);
        self.record_session_action(result);
    }

    fn write_pin(&mut self, pin: PinKey) {
        if !self.require_connection() {
            return;
        }
        let result = self.session.write_pin(pin);
        self.record_session_action(result);
    }

    fn toggle_listener(&mut self, pin: PinKey) {
        if !self.require_connection() {
            return;
        }
        let result = self.session.toggle_listener(pin);
        self.record_session_action(result);
    }

    fn apply_bulk_mode(&mut self) {
        if !self.require_connection() {
            return;
        }
        if !self.bulk_modes().contains(&self.bulk_mode) {
            self.device_status = "No eligible pins in selected scope".into();
            return;
        }
        let result = self.session.apply_mode(
            self.selected_route_key(),
            self.bulk_scope.target,
            self.bulk_mode,
            self.overwrite,
        );
        if self.record_session_action(result) == 0 {
            self.device_status = "No eligible pins in selected scope".into();
        }
    }

    fn read_scope(&mut self, target: RoutedTarget) {
        if !self.require_connection() {
            return;
        }
        let result = self.session.read_scope(self.selected_route_key(), target);
        self.record_session_action(result);
    }

    fn set_listener_scope(&mut self, target: RoutedTarget, enabled: bool) {
        if !self.require_connection() {
            return;
        }
        let result = self
            .session
            .set_listener_scope(self.selected_route_key(), target, enabled);
        self.record_session_action(result);
    }

    fn set_scope_level(&mut self, target: RoutedTarget, level: Level) {
        if !self.require_connection() {
            return;
        }
        let result = self
            .session
            .set_scope_level(self.selected_route_key(), target, level);
        self.record_session_action(result);
    }

    pub(super) fn selected_route_key(&self) -> RouteKey {
        self.selected_route.key
    }

    pub(super) fn bulk_modes(&self) -> Vec<Mode> {
        let route = self.selected_route_key();
        let pins = self.session.target_pins(route, self.bulk_scope.target);
        MODES
            .into_iter()
            .filter(|mode| {
                pins.iter().any(|&pin| {
                    self.session
                        .pin_info(pin)
                        .is_some_and(|info| mode.supported_by(info.capabilities))
                })
            })
            .collect()
    }

    fn normalize_bulk_mode(&mut self) {
        let modes = self.bulk_modes();
        if !modes.contains(&self.bulk_mode)
            && let Some(mode) = modes.first().copied()
        {
            self.bulk_mode = mode;
        }
    }

    fn sync_route_ui(&mut self) {
        let route = self.selected_route_key();
        self.bulk_scopes = std::iter::once(ScopeChoice::all())
            .chain(self.session.banks(route).map(|(bank, token)| ScopeChoice {
                target: RoutedTarget::Bank(bank),
                label: token.to_owned(),
            }))
            .collect();
        self.bulk_scope = ScopeChoice::all();
        self.bank_groups = self.bank_groups_for(route);
        self.bank_group = 0;
        self.confirm_set = None;
        self.normalize_bulk_mode();
    }

    fn bank_groups_for(&self, route: RouteKey) -> Vec<BankGroup> {
        let discovered: Vec<_> = self
            .session
            .banks(route)
            .map(|(key, token)| (key, token.to_owned()))
            .collect();
        let Some(specs) = self
            .route_layout
            .filter(|layout| layout.route == route)
            .map(|layout| layout.groups)
        else {
            return discovered
                .into_iter()
                .map(|(bank, label)| BankGroup {
                    label,
                    banks: vec![bank],
                })
                .collect();
        };

        let mut used = Vec::new();
        let mut groups = Vec::new();
        for spec in specs {
            let banks: Vec<_> = spec
                .banks
                .iter()
                .filter_map(|token| {
                    discovered
                        .iter()
                        .find(|(_, label)| label == token)
                        .map(|(bank, _)| *bank)
                })
                .collect();
            if !banks.is_empty() {
                used.extend(banks.iter().copied());
                groups.push(BankGroup {
                    label: spec.label.to_owned(),
                    banks,
                });
            }
        }
        groups.extend(
            discovered
                .into_iter()
                .filter(|(bank, _)| !used.contains(bank))
                .map(|(bank, label)| BankGroup {
                    label,
                    banks: vec![bank],
                }),
        );
        groups
    }

    fn record_session_action(&mut self, result: Result<Vec<String>, String>) -> usize {
        match result {
            Ok(lines) => {
                let count = lines.len();
                for line in lines {
                    self.push_log(format!("TX {line}"));
                }
                count
            }
            Err(error) => {
                self.error = Some(error);
                0
            }
        }
    }

    fn connected(&mut self, port: String) {
        self.connected_port = Some(port);
        self.device_status = "Connected".into();
        self.error = None;
        self.sync_route_ui();
    }

    fn send_request(&mut self, request: Request) {
        if !self.require_connection() {
            return;
        }
        match self.session.send(self.selected_route_key(), request) {
            Ok(line) => self.push_log(format!("TX {line}")),
            Err(error) => self.error = Some(error),
        }
    }

    fn send_raw(&mut self) {
        if self.raw_input.is_empty() {
            return;
        }
        if !self.require_connection() {
            return;
        }

        let line = std::mem::take(&mut self.raw_input);
        self.command_history.push_back(line.clone());
        if self.command_history.len() > MAX_COMMAND_HISTORY {
            self.command_history.pop_front();
        }
        self.history_index = None;
        match self.session.send_raw(&line) {
            Ok(()) => self.push_log(format!("TX {line}")),
            Err(error) => self.error = Some(error),
        }
    }

    fn require_connection(&mut self) -> bool {
        if self.connected_port.is_some() {
            true
        } else {
            self.error = Some("No serial device connected".into());
            false
        }
    }

    fn drain_io(&mut self) {
        self.session.poll_listener_updates();
        for _ in 0..MAX_IO_EVENTS_PER_TICK {
            let Some(event) = self.session.next_event() else {
                break;
            };
            match event {
                ConnectionEvent::Connected(port) => {
                    self.connected(port);
                }
                ConnectionEvent::Disconnected(reason) => {
                    self.connected_port = None;
                    self.device_status = "Disconnected".into();
                    self.sync_route_ui();
                    self.error = reason;
                }
                ConnectionEvent::Received { line, event } => {
                    self.push_log(format!("RX {line}"));
                    match event {
                        Ok(event) => self.handle_device_event(event),
                        Err(error) => self.error = Some(error),
                    }
                }
                ConnectionEvent::ListenerValues(values) => {
                    for value in values {
                        self.push_log(format!("RX {}", value.line()));
                        if value.coalesced != 0 {
                            self.push_log(format!(
                                "RX ({} intermediate listener updates coalesced)",
                                value.coalesced
                            ));
                        }
                        self.handle_device_event(DeviceEvent::PinValue {
                            pin: value.pin,
                            level: value.level,
                        });
                    }
                }
                ConnectionEvent::IoError(error) => self.error = Some(error),
            }
        }
    }

    fn handle_device_event(&mut self, event: DeviceEvent) {
        match event {
            DeviceEvent::Hello { route } => {
                self.device_status = format!("{} replied HII", self.session.route_name(route));
            }
            DeviceEvent::Status { route, identity } => {
                self.device_status = format!("{}: {identity}", self.session.route_name(route));
            }
            DeviceEvent::Version { route, version } => {
                self.device_status = format!(
                    "{} protocol version {version}",
                    self.session.route_name(route)
                );
            }
            DeviceEvent::Help { route, command } => {
                self.device_status =
                    format!("{} supports {command}", self.session.route_name(route));
            }
            DeviceEvent::MapReady { route } => {
                if route == self.selected_route_key() {
                    self.sync_route_ui();
                }
                self.device_status = format!(
                    "{} map: {} banks, {} pins",
                    self.session.route_name(route),
                    self.session.banks(route).count(),
                    self.session.pins(route).count()
                );
            }
            DeviceEvent::Ack { route: _, sent } => {
                if let Some(line) = sent {
                    self.push_log(format!("TX {line}"));
                }
            }
            DeviceEvent::PinValue { .. } => {}
            DeviceEvent::PinState { pin, what, value } => {
                self.device_status =
                    format!("{} {what:?}: {value:?}", self.routed_pin_display(pin));
            }
            DeviceEvent::DeviceError {
                route: _,
                source,
                error,
            } => {
                self.error = Some(match error {
                    ResponseError::BadPacket => {
                        format!("{source} rejected a malformed packet")
                    }
                    ResponseError::Target {
                        target: pin,
                        reason,
                    } => format!("{}: {reason:?}", self.routed_pin_display(pin)),
                    ResponseError::NoRoute { destination } => {
                        format!("{source}: no route to {destination}")
                    }
                    ResponseError::RouteBusy { next_hop } => {
                        format!("{source}: route {next_hop} is busy")
                    }
                    ResponseError::RouteDown { next_hop } => {
                        format!("{source}: route {next_hop} is down")
                    }
                });
            }
            DeviceEvent::Unknown { route } => {
                self.error = Some(format!("{} returned IDK", self.session.route_name(route)));
            }
            DeviceEvent::Bye { route } => {
                self.device_status =
                    format!("{} reset acknowledged", self.session.route_name(route));
            }
            DeviceEvent::Untracked => {}
        }
    }

    fn routed_pin_display(&self, pin: PinKey) -> String {
        self.session
            .pin_info(pin)
            .map_or_else(|| "unknown pin".into(), ToString::to_string)
    }

    fn push_log(&mut self, text: String) {
        self.log.push(text);
    }

    fn snap_log(&self) -> Task<Message> {
        if self.autoscroll {
            iced::widget::operation::snap_to_end(self.log_scroll.clone())
        } else {
            Task::none()
        }
    }

    fn history_previous(&mut self) {
        if self.command_history.is_empty() {
            return;
        }
        let index = self
            .history_index
            .map_or(self.command_history.len() - 1, |index| {
                index.saturating_sub(1)
            });
        self.history_index = Some(index);
        self.raw_input.clone_from(&self.command_history[index]);
    }

    fn history_next(&mut self) {
        let Some(index) = self.history_index else {
            return;
        };
        if index + 1 < self.command_history.len() {
            self.history_index = Some(index + 1);
            self.raw_input.clone_from(&self.command_history[index + 1]);
        } else {
            self.history_index = None;
            self.raw_input.clear();
        }
    }
}
fn load_ports() -> Task<Message> {
    Task::perform(
        async { DeviceSession::available_ports() },
        Message::PortsLoaded,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use da_vinci_protocol::PinCapabilities;

    fn install_sam_map(app: &mut App) {
        let sam = app.session.route_key("SAM").unwrap();
        let banks = vec!["PIOA", "PIOB", "PIOE", "PIOC", "PIOD"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        let pins = vec![
            ("PA00".into(), 0, 0, PinCapabilities::GPIO),
            ("PB00".into(), 1, 0, PinCapabilities::GPIO),
            ("PE00".into(), 2, 0, PinCapabilities::INPUT_PULLUP),
            ("PC00".into(), 3, 0, PinCapabilities::GPIO),
            ("PD00".into(), 4, 0, PinCapabilities::GPIO),
        ];
        app.session.install_map_for_test(sam, banks, pins);
    }

    fn connected_app() -> App {
        let mut app = App::with_routes(&ROUTES);
        app.connected_port = Some("test".into());
        install_sam_map(&mut app);
        app.sync_route_ui();
        app
    }

    fn route(app: &App, label: &str) -> RouteChoice {
        app.routes
            .iter()
            .find(|route| route.label == label)
            .unwrap()
            .clone()
    }

    fn scope(app: &App, token: &str) -> ScopeChoice {
        app.bulk_scopes
            .iter()
            .find(|scope| scope.label == token)
            .unwrap()
            .clone()
    }

    fn last_log(app: &App) -> &str {
        app.log.last_text().unwrap()
    }

    #[test]
    fn connection_does_not_start_pin_map_discovery() {
        let mut app = App::with_routes(&ROUTES);

        app.connected("test".into());

        assert_eq!(app.device_status, "Connected");
        assert!(app.log.is_empty());
        app.send_request(Request::Hello);
        assert_eq!(last_log(&app), "TX 001 SAM HAI");
    }

    #[test]
    fn manual_pin_map_uses_selected_route_and_does_not_block_handshake() {
        let mut app = App::with_routes(&ROUTES);
        app.connected("test".into());
        let lpc = route(&app, "LPC");
        let _ = app.update(Message::RouteSelected(lpc));

        let _ = app.update(Message::PinMap);
        assert_eq!(last_log(&app), "TX 001 LPC MAP");

        let _ = app.update(Message::Handshake);
        assert_eq!(last_log(&app), "TX 002 LPC HAI");

        let _ = app.update(Message::PinMap);
        assert_eq!(
            app.error.as_deref(),
            Some("MAP for LPC is already in progress")
        );
        assert_eq!(last_log(&app), "TX 002 LPC HAI");
    }

    #[test]
    fn typed_protocol_diagnostics_use_selected_route() {
        let mut app = App::with_routes(&ROUTES);
        app.connected("test".into());
        let lpc = route(&app, "LPC");
        let _ = app.update(Message::RouteSelected(lpc));

        let _ = app.update(Message::Version);
        assert_eq!(last_log(&app), "TX 001 LPC VER");

        let _ = app.update(Message::Help);
        assert_eq!(last_log(&app), "TX 002 LPC HLP");
    }

    #[test]
    fn bulk_controls_send_selected_discovered_scope() {
        let mut app = connected_app();
        let port_c = scope(&app, "PIOC");
        app.bulk_scope = port_c.clone();
        app.bulk_mode = Mode::InputPullup;
        app.overwrite = true;

        app.apply_bulk_mode();
        assert_eq!(last_log(&app), "TX 001 SAM DIR PIOC IN OK?");

        app.set_listener_scope(port_c.target, true);
        assert_eq!(last_log(&app), "TX 002 SAM LSN PIOC ON OK?");

        app.read_scope(port_c.target);
        assert_eq!(last_log(&app), "TX 003 SAM GET PIOC OK?");
    }

    #[test]
    fn sam_layout_groups_discovered_banks_without_owning_topology() {
        let app = connected_app();
        let labels: Vec<_> = app
            .bank_groups
            .iter()
            .map(|group| group.label.as_str())
            .collect();
        assert_eq!(labels, ["PIOA", "PIOB + PIOE", "PIOC", "PIOD"]);
        assert_eq!(app.bank_groups[1].banks.len(), 2);
        assert_eq!(app.bulk_scopes.len(), 6);
    }

    #[test]
    fn route_switch_preserves_session_owned_pin_state() {
        let mut app = connected_app();
        let sam = app.session.route_key("SAM").unwrap();
        let lpc = app.session.route_key("LPC").unwrap();
        app.session.install_map_for_test(
            lpc,
            vec!["PIO2".into()],
            vec![("PIO2_3".into(), 0, 3, PinCapabilities::INPUT)],
        );
        let sam_pin = app.session.pin_key(sam, "PA00").unwrap();
        app.change_mode(sam_pin, Mode::Input);
        assert_eq!(
            app.session.pin_state(sam_pin).unwrap().target_mode,
            Some(Mode::Input)
        );

        let _ = app.update(Message::RouteSelected(route(&app, "LPC")));
        assert_eq!(app.selected_route_key(), lpc);
        assert_eq!(
            app.session.pin_state(sam_pin).unwrap().target_mode,
            Some(Mode::Input)
        );

        let _ = app.update(Message::RouteSelected(route(&app, "SAM")));
        assert_eq!(app.selected_route_key(), sam);
        assert_eq!(
            app.session.pin_state(sam_pin).unwrap().target_mode,
            Some(Mode::Input)
        );
    }

    #[test]
    fn synthetic_route_uses_discovery_for_groups_scopes_and_commands() {
        let mut app = App::with_routes(&["SAM", "LPC", "AUX"]);
        app.connected_port = Some("test".into());
        let aux = app.session.route_key("AUX").unwrap();
        app.session.install_map_for_test(
            aux,
            vec!["GPIOX".into(), "GPIOY".into()],
            vec![
                ("X0".into(), 0, 0, PinCapabilities::INPUT_PULLUP),
                ("Y7".into(), 1, 7, PinCapabilities::OUTPUT),
            ],
        );

        let _ = app.update(Message::RouteSelected(route(&app, "AUX")));
        assert_eq!(app.selected_route_key(), aux);
        assert_eq!(
            app.bank_groups
                .iter()
                .map(|group| group.label.as_str())
                .collect::<Vec<_>>(),
            ["GPIOX", "GPIOY"]
        );
        assert_eq!(
            app.bulk_scopes
                .iter()
                .map(|scope| scope.label.as_str())
                .collect::<Vec<_>>(),
            ["ALL", "GPIOX", "GPIOY"]
        );
        let _ = view::view(&app);

        app.bulk_scope = scope(&app, "GPIOY");
        app.bulk_mode = Mode::Output;
        app.overwrite = true;
        app.apply_bulk_mode();
        assert_eq!(last_log(&app), "TX 001 AUX DIR GPIOY OUT OK?");
    }

    #[test]
    fn intermediate_route_error_is_visible_independent_of_selection() {
        let mut app = connected_app();
        let lpc = app.session.route_key("LPC").unwrap();
        app.handle_device_event(DeviceEvent::DeviceError {
            route: lpc,
            source: "SAM".into(),
            error: ResponseError::RouteDown {
                next_hop: "LPC".into(),
            },
        });
        assert_eq!(app.error.as_deref(), Some("SAM: route LPC is down"));
    }

    #[test]
    fn bulk_set_waits_for_confirmation() {
        let mut app = connected_app();

        let _ = app.update(Message::BulkSet(Level::High));
        assert!(app.log.is_empty());
        assert_eq!(
            app.confirm_set
                .as_ref()
                .map(|(scope, level)| (&scope.target, *level)),
            Some((&RoutedTarget::All, Level::High))
        );

        let _ = app.update(Message::BulkSetConfirm);
        assert_eq!(last_log(&app), "TX 001 SAM SET ALL HIGH OK?");
    }

    #[test]
    fn group_selection_does_not_change_bulk_scope() {
        let mut app = connected_app();
        let selected = scope(&app, "PIOC");
        app.bulk_scope = selected.clone();

        let _ = app.update(Message::GroupSelected(3));

        assert_eq!(app.bank_group, 3);
        assert_eq!(app.bulk_scope, selected);
    }
}
