use std::{
    collections::VecDeque,
    io::{self, Read, Write},
    sync::mpsc::{self, Receiver, Sender, SyncSender},
    thread,
    time::Duration,
};

use da_vinci_protocol::{
    DecodeError, DecodeErrorKind, DecodedResponse, Frame, Level, LineBuffer, LineError,
    MAX_PACKET_LEN, Message, Packet, RawMessage, RequestId, Response as ProtocolResponse,
};

const EVENT_QUEUE_CAPACITY: usize = 1_024;

pub(super) type WireResponse = ProtocolResponse<String, String>;
pub(super) type OwnedResponse = Message<String, WireResponse>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ListenerKey {
    pub(super) route: usize,
    pub(super) pin: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ListenerValue {
    pub(super) line: Frame,
    pub(super) id: RequestId,
    pub(super) key: ListenerKey,
    pub(super) level: Level,
    pub(super) coalesced: u32,
}

pub(super) struct SerialIo {
    commands: Sender<IoCommand>,
    events: Receiver<IoEvent>,
}

impl SerialIo {
    pub(super) fn spawn() -> Self {
        let (command_tx, command_rx) = mpsc::channel();
        let (event_tx, event_rx) = mpsc::sync_channel(EVENT_QUEUE_CAPACITY);
        thread::spawn(move || io_worker(command_rx, event_tx));
        Self {
            commands: command_tx,
            events: event_rx,
        }
    }

    pub(super) fn available_ports() -> Result<Vec<String>, String> {
        serialport::available_ports()
            .map(|ports| ports.into_iter().map(|port| port.port_name).collect())
            .map_err(|error| error.to_string())
    }

    pub(super) fn connect(&self, name: String) -> Result<(), String> {
        self.send(IoCommand::Connect(name))
    }

    pub(super) fn disconnect(&self) -> Result<(), String> {
        self.send(IoCommand::Disconnect)
    }

    pub(super) fn write(&self, frame: Frame) -> Result<(), String> {
        self.send(IoCommand::Write(frame))
    }

    pub(super) fn set_listeners(&self, routes: Vec<ListenerRoute>) {
        let _ = self.commands.send(IoCommand::Listeners(routes));
    }

    pub(super) fn drain_listeners(&self) {
        let _ = self.commands.send(IoCommand::DrainListeners);
    }

    pub(super) fn next_event(&self) -> Option<IoEvent> {
        match self.events.try_recv() {
            Ok(event) => Some(event),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => {
                Some(IoEvent::Disconnected(Some("Serial worker stopped".into())))
            }
        }
    }

    fn send(&self, command: IoCommand) -> Result<(), String> {
        self.commands
            .send(command)
            .map_err(|_| "Serial worker stopped".into())
    }

    #[cfg(test)]
    pub(super) fn stop_for_test(&mut self) {
        let (commands, receiver) = mpsc::channel();
        drop(receiver);
        self.commands = commands;
    }
}

#[derive(Clone)]
pub(super) struct ListenerPin {
    pub(super) key: ListenerKey,
    pub(super) token: Box<[u8]>,
    pub(super) id: RequestId,
}

#[derive(Clone)]
pub(super) struct ListenerRoute {
    pub(super) name: Box<[u8]>,
    pub(super) pin_count: usize,
    pub(super) pins: Vec<ListenerPin>,
}

enum IoCommand {
    Connect(String),
    Disconnect,
    Write(Frame),
    Listeners(Vec<ListenerRoute>),
    DrainListeners,
}

pub(super) enum IoEvent {
    Connected(String),
    Disconnected(Option<String>),
    Line {
        line: Frame,
        packet: Result<OwnedResponse, DecodeError>,
    },
    ListenerValues(Vec<ListenerValue>),
    Error(String),
}

struct IoState {
    port: Option<Box<dyn serialport::SerialPort>>,
    reader: LineBuffer,
    writes: VecDeque<Frame>,
    write_offset: usize,
    listeners: Vec<ListenerRoute>,
    listener_updates: Vec<Vec<Option<ListenerValue>>>,
}

impl IoState {
    fn new() -> Self {
        Self {
            port: None,
            reader: LineBuffer::new(),
            writes: VecDeque::new(),
            write_offset: 0,
            listeners: Vec::new(),
            listener_updates: Vec::new(),
        }
    }

    fn clear_listeners(&mut self) {
        self.listeners.clear();
        self.listener_updates.clear();
    }

    fn clear_stream_state(&mut self) {
        self.writes.clear();
        self.write_offset = 0;
        self.reader.clear();
        self.clear_listeners();
    }

    fn become_disconnected(&mut self) {
        self.port = None;
        self.clear_stream_state();
    }
}

fn io_worker(commands: Receiver<IoCommand>, events: SyncSender<IoEvent>) {
    let mut state = IoState::new();
    let mut buffer = [0u8; 64];

    loop {
        if state.port.is_none() {
            let Ok(command) = commands.recv() else {
                return;
            };
            handle_io_command(command, &mut state, &events);
        } else {
            loop {
                match commands.try_recv() {
                    Ok(command) => handle_io_command(command, &mut state, &events),
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => return,
                }
            }
        }

        if state.port.is_none() {
            continue;
        }

        if let Some(frame) = state.writes.front() {
            let bytes = frame.as_ref();
            let result = state
                .port
                .as_mut()
                .expect("connected serial port")
                .write(&bytes[state.write_offset..]);
            match result {
                Ok(written) => {
                    state.write_offset += written;
                    if state.write_offset == bytes.len() {
                        state.writes.pop_front();
                        state.write_offset = 0;
                    }
                }
                Err(error) if transient_io_error(&error) => {}
                Err(error) => {
                    state.become_disconnected();
                    let _ = events.send(IoEvent::Disconnected(Some(format!(
                        "Serial write failed: {error}"
                    ))));
                    continue;
                }
            }
        }

        match state
            .port
            .as_mut()
            .expect("connected serial port")
            .read(&mut buffer)
        {
            Ok(count) => {
                for &byte in &buffer[..count] {
                    match state.reader.push(byte) {
                        Ok(Some(line)) => {
                            let line = Frame::try_from(line)
                                .expect("line buffer enforces protocol frame capacity");
                            route_line(line, &events, &mut state);
                        }
                        Ok(None) => {}
                        Err(LineError::TooLong) => {
                            let _ = events.send(IoEvent::Error(format!(
                                "Incoming serial line exceeded {} bytes; discarded",
                                MAX_PACKET_LEN - 1
                            )));
                        }
                    }
                }
            }
            Err(error) if transient_io_error(&error) => {}
            Err(error) => {
                state.become_disconnected();
                let _ = events.send(IoEvent::Disconnected(Some(format!(
                    "Serial read failed: {error}"
                ))));
            }
        }
    }
}

fn route_line(wire_line: Frame, events: &SyncSender<IoEvent>, state: &mut IoState) {
    let decoded =
        RawMessage::try_from(&wire_line).and_then(Message::<&[u8], DecodedResponse<'_>>::try_from);
    match decoded {
        Ok(Message {
            route: source,
            packet:
                Packet {
                    id,
                    body: ProtocolResponse::Value { target, level },
                },
        }) => {
            if let Some(pin) = active_listener(&state.listeners, source, id, target) {
                coalesce_listener_update(&mut state.listener_updates, pin, wire_line, id, level);
            } else {
                let _ = events.send(IoEvent::Line {
                    line: wire_line,
                    packet: own_response(Message {
                        route: source,
                        packet: Packet {
                            id,
                            body: ProtocolResponse::Value { target, level },
                        },
                    }),
                });
            }
        }
        Ok(message) => {
            let _ = events.send(IoEvent::Line {
                line: wire_line,
                packet: own_response(message),
            });
        }
        Err(error) => {
            let _ = events.send(IoEvent::Line {
                line: wire_line,
                packet: Err(error),
            });
        }
    }
}

fn own_response(
    message: Message<&[u8], DecodedResponse<'_>>,
) -> Result<OwnedResponse, DecodeError> {
    let packet = message.packet;
    let malformed = || DecodeError {
        id: Some(packet.id),
        kind: DecodeErrorKind::Malformed,
    };
    let body = packet.body.try_map(
        |target| {
            core::str::from_utf8(target)
                .map(str::to_owned)
                .map_err(|_| malformed())
        },
        |data| {
            core::str::from_utf8(data)
                .map(str::to_owned)
                .map_err(|_| malformed())
        },
    )?;
    Ok(OwnedResponse {
        route: String::from_utf8_lossy(message.route).into_owned(),
        packet: Packet {
            id: packet.id,
            body,
        },
    })
}

fn active_listener(
    routes: &[ListenerRoute],
    source: &[u8],
    id: RequestId,
    target: &[u8],
) -> Option<ListenerKey> {
    routes
        .iter()
        .find(|route| route.name.as_ref() == source)?
        .pins
        .iter()
        .find(|pin| pin.id == id && pin.token.as_ref() == target)
        .map(|pin| pin.key)
}

fn listener_is_configured(routes: &[ListenerRoute], key: ListenerKey, id: RequestId) -> bool {
    routes
        .iter()
        .any(|route| route.pins.iter().any(|pin| pin.key == key && pin.id == id))
}

fn coalesce_listener_update(
    updates: &mut [Vec<Option<ListenerValue>>],
    key: ListenerKey,
    line: Frame,
    id: RequestId,
    level: Level,
) {
    let slot = &mut updates[key.route][key.pin];
    let coalesced = slot.map_or(0, |previous| previous.coalesced.saturating_add(1));
    *slot = Some(ListenerValue {
        line,
        id,
        key,
        level,
        coalesced,
    });
}

fn handle_io_command(command: IoCommand, state: &mut IoState, events: &SyncSender<IoEvent>) {
    match command {
        IoCommand::Connect(name) => {
            state.clear_stream_state();
            match serialport::new(&name, 115_200)
                .timeout(Duration::from_millis(20))
                .open()
            {
                Ok(opened) => {
                    state.port = Some(opened);
                    let _ = events.send(IoEvent::Connected(name));
                }
                Err(error) => {
                    state.become_disconnected();
                    let _ = events.send(IoEvent::Error(format!("Could not open {name}: {error}")));
                }
            }
        }
        IoCommand::Disconnect => {
            state.become_disconnected();
            let _ = events.send(IoEvent::Disconnected(None));
        }
        IoCommand::Write(frame) => {
            if state.port.is_some() {
                state.writes.push_back(frame);
            }
        }
        IoCommand::Listeners(routes) => {
            let mut updates: Vec<Vec<Option<ListenerValue>>> = routes
                .iter()
                .map(|route| vec![None; route.pin_count])
                .collect();
            for update in state
                .listener_updates
                .iter_mut()
                .flat_map(|route| route.iter_mut())
                .filter_map(Option::take)
            {
                if listener_is_configured(&routes, update.key, update.id) {
                    updates[update.key.route][update.key.pin] = Some(update);
                }
            }
            state.listener_updates = updates;
            state.listeners = routes;
        }
        IoCommand::DrainListeners => {
            let updates: Vec<ListenerValue> = state
                .listener_updates
                .iter_mut()
                .flat_map(|route| route.iter_mut())
                .filter_map(Option::take)
                .collect();
            if !updates.is_empty() {
                let _ = events.send(IoEvent::ListenerValues(updates));
            }
        }
    }
}

fn transient_io_error(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(bytes: &[u8]) -> Frame {
        Frame::try_from(bytes).unwrap()
    }

    fn request_id(raw: u16) -> RequestId {
        RequestId::new(raw).unwrap()
    }

    fn listener_route(id: RequestId) -> ListenerRoute {
        ListenerRoute {
            name: b"SAM".as_slice().into(),
            pin_count: 2,
            pins: vec![ListenerPin {
                key: ListenerKey { route: 0, pin: 0 },
                token: b"PA00".as_slice().into(),
                id,
            }],
        }
    }

    #[test]
    fn listener_updates_coalesce_by_compact_key() {
        let id = request_id(8);
        let key = ListenerKey { route: 0, pin: 0 };
        let routes = vec![listener_route(id)];
        let mut updates = vec![vec![None; 2]];

        assert_eq!(active_listener(&routes, b"SAM", id, b"PA00"), Some(key));
        coalesce_listener_update(
            &mut updates,
            key,
            frame(b"008 SAM HYG PA00 LOW <3"),
            id,
            Level::Low,
        );
        coalesce_listener_update(
            &mut updates,
            key,
            frame(b"008 SAM HYG PA00 HIGH <3"),
            id,
            Level::High,
        );

        let update = updates[0][0].unwrap();
        assert_eq!(update.coalesced, 1);
        assert_eq!(update.level, Level::High);
        assert_eq!(update.key, key);
    }

    #[test]
    fn listener_snapshot_keeps_only_still_configured_updates() {
        let old = request_id(8);
        let new = request_id(9);
        let key = ListenerKey { route: 0, pin: 0 };
        let (events, received) = mpsc::sync_channel(2);
        let mut state = IoState::new();

        handle_io_command(
            IoCommand::Listeners(vec![listener_route(old)]),
            &mut state,
            &events,
        );
        coalesce_listener_update(
            &mut state.listener_updates,
            key,
            frame(b"008 SAM HYG PA00 HIGH <3"),
            old,
            Level::High,
        );
        handle_io_command(
            IoCommand::Listeners(vec![listener_route(new)]),
            &mut state,
            &events,
        );
        handle_io_command(IoCommand::DrainListeners, &mut state, &events);
        assert!(received.try_recv().is_err());

        handle_io_command(
            IoCommand::Listeners(vec![listener_route(new)]),
            &mut state,
            &events,
        );
        coalesce_listener_update(
            &mut state.listener_updates,
            key,
            frame(b"009 SAM HYG PA00 LOW <3"),
            new,
            Level::Low,
        );
        let mut expanded = listener_route(new);
        expanded.pins.push(ListenerPin {
            key: ListenerKey { route: 0, pin: 1 },
            token: b"PA01".as_slice().into(),
            id: request_id(10),
        });
        handle_io_command(IoCommand::Listeners(vec![expanded]), &mut state, &events);
        handle_io_command(IoCommand::DrainListeners, &mut state, &events);
        let Ok(IoEvent::ListenerValues(values)) = received.try_recv() else {
            panic!("expected preserved listener update");
        };
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].key, key);
        assert_eq!(values[0].level, Level::Low);
    }

    #[test]
    fn transient_serial_errors_are_retryable() {
        for kind in [
            io::ErrorKind::TimedOut,
            io::ErrorKind::WouldBlock,
            io::ErrorKind::Interrupted,
        ] {
            assert!(transient_io_error(&io::Error::from(kind)));
        }
        assert!(!transient_io_error(&io::Error::from(
            io::ErrorKind::BrokenPipe
        )));
    }

    #[test]
    fn stale_write_after_disconnect_is_dropped_without_error() {
        let (events, received) = mpsc::sync_channel(1);
        let mut state = IoState::new();
        handle_io_command(
            IoCommand::Write(frame(b"001 SAM HAI\n")),
            &mut state,
            &events,
        );
        assert!(state.writes.is_empty());
        assert!(received.try_recv().is_err());
    }
}
