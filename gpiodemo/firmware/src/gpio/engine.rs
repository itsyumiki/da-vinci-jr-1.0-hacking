use da_vinci_protocol::{
    Command, DecodedRequest, Direction, Level, PROTOCOL_VERSION, Packet, Query, QueryValue,
    RequestId, Response, ResponseError, TargetError, Toggle,
};

use super::{
    GpioHal, PinMode,
    map::{MAX_BANKS, MAX_PINS, PinId, PinMap, Target},
};

type FirmwareResponse = Response<&'static [u8], &'static [u8]>;

#[derive(Clone, Copy)]
enum PinState {
    Unset,
    Input {
        pull_up: bool,
        listener: Option<RequestId>,
        previous: Level,
    },
    Output,
}

#[derive(Clone, Copy)]
enum BulkKind {
    Values(Target),
    States(Target, Query),
    Map,
    Help,
}

#[derive(Clone, Copy)]
struct BulkResponse {
    id: RequestId,
    next: usize,
    kind: BulkKind,
}

pub(crate) struct Firmware {
    identity: &'static [u8],
    pins: [PinState; MAX_PINS],
    bulk: Option<BulkResponse>,
    listener_cursor: usize,
}

impl Firmware {
    pub(crate) const fn new(identity: &'static [u8]) -> Self {
        Self {
            identity,
            pins: [PinState::Unset; MAX_PINS],
            bulk: None,
            listener_cursor: 0,
        }
    }

    pub(crate) fn handle<G: GpioHal>(
        &mut self,
        packet: Packet<DecodedRequest<'_>>,
        gpio: &mut G,
    ) -> Packet<FirmwareResponse> {
        let map = gpio.pin_map();
        let body = match packet.body {
            DecodedRequest::Hello => Response::Hello,
            DecodedRequest::Status => Response::Status {
                identity: self.identity,
            },
            DecodedRequest::Map => return self.begin_bulk(packet.id, BulkKind::Map, gpio),
            DecodedRequest::Version => Response::Version {
                version: PROTOCOL_VERSION,
            },
            DecodedRequest::Help => return self.begin_bulk(packet.id, BulkKind::Help, gpio),
            DecodedRequest::Direction { target, direction } => {
                map.resolve(target).map_or_else(bad_packet, |target| {
                    self.set_direction(map, target, direction, gpio)
                })
            }
            DecodedRequest::Get { target } => {
                let Some(target) = map.resolve(target) else {
                    return Packet {
                        id: packet.id,
                        body: bad_packet(),
                    };
                };
                if let Target::Pin(pin) = target {
                    match self.initialized(map, pin) {
                        Ok(()) => Response::Value {
                            target: map.pin(pin).token.as_bytes(),
                            level: read_pin(map, gpio, pin),
                        },
                        Err(error) => error,
                    }
                } else {
                    return self.begin_bulk(packet.id, BulkKind::Values(target), gpio);
                }
            }
            DecodedRequest::Set { target, level } => {
                map.resolve(target).map_or_else(bad_packet, |target| {
                    self.set_level(map, target, level, gpio)
                })
            }
            DecodedRequest::Pullup { target, state } => {
                map.resolve(target).map_or_else(bad_packet, |target| {
                    self.set_pull_up(map, target, state == Toggle::On, gpio)
                })
            }
            DecodedRequest::Listen { target, state } => {
                map.resolve(target).map_or_else(bad_packet, |target| {
                    self.set_listening(map, target, state == Toggle::On, packet.id, gpio)
                })
            }
            DecodedRequest::Query { target, what } => {
                let Some(target) = map.resolve(target) else {
                    return Packet {
                        id: packet.id,
                        body: bad_packet(),
                    };
                };
                if let Target::Pin(pin) = target {
                    match supported(map, pin) {
                        Ok(()) => Response::State {
                            target: map.pin(pin).token.as_bytes(),
                            what,
                            value: self.query(pin, what),
                        },
                        Err(error) => error,
                    }
                } else {
                    return self.begin_bulk(packet.id, BulkKind::States(target, what), gpio);
                }
            }
            DecodedRequest::Bye => {
                self.reset(gpio);
                Response::Bye
            }
        };
        Packet {
            id: packet.id,
            body,
        }
    }

    fn begin_bulk<G: GpioHal>(
        &mut self,
        id: RequestId,
        kind: BulkKind,
        gpio: &G,
    ) -> Packet<FirmwareResponse> {
        self.bulk = Some(BulkResponse { id, next: 0, kind });
        self.poll_bulk(gpio)
            .expect("new bulk response always yields a packet")
    }

    pub(crate) fn poll_bulk<G: GpioHal>(&mut self, gpio: &G) -> Option<Packet<FirmwareResponse>> {
        let BulkResponse { id, mut next, kind } = self.bulk?;

        if matches!(kind, BulkKind::Help) {
            let Some(&command) = Command::ALL.get(next) else {
                self.bulk = None;
                return Some(Packet {
                    id,
                    body: Response::Ack,
                });
            };
            next += 1;
            self.bulk = Some(BulkResponse { id, next, kind });
            return Some(Packet {
                id,
                body: Response::Help { command },
            });
        }

        let map = gpio.pin_map();

        if matches!(kind, BulkKind::Map) {
            let body = if next < map.banks().len() {
                Response::MapBank {
                    bank: map.banks()[next].as_bytes(),
                }
            } else if let Some(info) = map.pins().get(next - map.banks().len()) {
                Response::MapPin {
                    target: info.token.as_bytes(),
                    package_pin: info.package_pin,
                    bank: map.bank(info.bank).as_bytes(),
                    bit: info.bit,
                    capabilities: info.capabilities,
                }
            } else {
                self.bulk = None;
                return Some(Packet {
                    id,
                    body: Response::Ack,
                });
            };
            next += 1;
            self.bulk = Some(BulkResponse { id, next, kind });
            return Some(Packet { id, body });
        }

        let target = match kind {
            BulkKind::Values(target) | BulkKind::States(target, _) => target,
            BulkKind::Map | BulkKind::Help => unreachable!(),
        };

        while next < map.pins().len() {
            let pin = map.pin_id(next);
            next += 1;
            let info = map.pin(pin);
            if !target_contains(map, target, pin) || !info.capabilities.available() {
                continue;
            }

            let body = match kind {
                BulkKind::Values(_) => {
                    if matches!(self.state(pin), PinState::Unset) {
                        continue;
                    }
                    Response::Value {
                        target: info.token.as_bytes(),
                        level: read_pin(map, gpio, pin),
                    }
                }
                BulkKind::States(_, what) => Response::State {
                    target: info.token.as_bytes(),
                    what,
                    value: self.query(pin, what),
                },
                BulkKind::Map | BulkKind::Help => unreachable!(),
            };
            self.bulk = Some(BulkResponse { id, next, kind });
            return Some(Packet { id, body });
        }

        self.bulk = None;
        Some(Packet {
            id,
            body: Response::Ack,
        })
    }

    pub(crate) fn poll_listener<G: GpioHal>(
        &mut self,
        gpio: &G,
    ) -> Option<Packet<FirmwareResponse>> {
        let map = gpio.pin_map();
        let pin_count = map.pins().len();
        if pin_count == 0 {
            return None;
        }

        let mut snapshots = [None; MAX_BANKS];
        for offset in 0..pin_count {
            let index = (self.listener_cursor + offset) % pin_count;
            let pin = map.pin_id(index);
            let PinState::Input {
                listener: Some(listener),
                previous,
                ..
            } = self.state(pin)
            else {
                continue;
            };
            let info = map.pin(pin);
            let snapshot =
                *snapshots[info.bank.index()].get_or_insert_with(|| gpio.read_bank(info.bank));
            let value = level_from_bank(snapshot, info.bit);
            if value == previous {
                continue;
            }
            if let PinState::Input { previous, .. } = self.state_mut(pin) {
                *previous = value;
            }
            self.listener_cursor = (index + 1) % pin_count;
            return Some(Packet {
                id: listener,
                body: Response::Value {
                    target: info.token.as_bytes(),
                    level: value,
                },
            });
        }
        None
    }

    fn set_direction<G: GpioHal>(
        &mut self,
        map: &PinMap,
        target: Target,
        direction: Direction,
        gpio: &mut G,
    ) -> FirmwareResponse {
        if let Target::Pin(pin) = target
            && !map.pin(pin).capabilities.supports_direction(direction)
        {
            return pin_error(map, pin, TargetError::Unavailable);
        }
        for pin in map
            .pins_for(target)
            .filter(|pin| map.pin(*pin).capabilities.supports_direction(direction))
        {
            self.set_direction_pin(map, pin, direction, gpio);
        }
        Response::Ack
    }

    fn set_direction_pin<G: GpioHal>(
        &mut self,
        map: &PinMap,
        pin: PinId,
        direction: Direction,
        gpio: &mut G,
    ) {
        match direction {
            Direction::Input => {
                let listener = match self.state(pin) {
                    PinState::Input { listener, .. } => listener,
                    PinState::Unset | PinState::Output => None,
                };
                gpio.configure(pin, PinMode::Input);
                self.pins[pin.index()] = PinState::Input {
                    pull_up: false,
                    listener,
                    previous: read_pin(map, gpio, pin),
                };
            }
            Direction::Output => {
                gpio.configure(
                    pin,
                    PinMode::Output {
                        initial: Level::Low,
                    },
                );
                self.pins[pin.index()] = PinState::Output;
            }
        }
    }

    fn set_level<G: GpioHal>(
        &mut self,
        map: &PinMap,
        target: Target,
        level: Level,
        gpio: &mut G,
    ) -> FirmwareResponse {
        if let Target::Pin(pin) = target
            && let Err(error) = self.initialized(map, pin)
        {
            return error;
        }
        let direct = matches!(target, Target::Pin(_));
        for pin in map
            .pins_for(target)
            .filter(|pin| map.pin(*pin).capabilities.output())
        {
            if direct || matches!(self.state(pin), PinState::Output) {
                gpio.write(pin, level);
            }
        }
        Response::Ack
    }

    fn set_pull_up<G: GpioHal>(
        &mut self,
        map: &PinMap,
        target: Target,
        enabled: bool,
        gpio: &mut G,
    ) -> FirmwareResponse {
        if let Target::Pin(pin) = target
            && let Err(error) = self.initialized(map, pin)
        {
            return error;
        }
        for pin in map.pins_for(target) {
            let info = map.pin(pin);
            let is_input = matches!(self.state(pin), PinState::Input { .. });
            if !info.capabilities.available() || (is_input && !info.capabilities.pull_up()) {
                continue;
            }
            self.set_pull_up_pin(map, pin, enabled, gpio);
        }
        Response::Ack
    }

    fn set_pull_up_pin<G: GpioHal>(
        &mut self,
        map: &PinMap,
        pin: PinId,
        enabled: bool,
        gpio: &mut G,
    ) {
        let PinState::Input {
            pull_up, previous, ..
        } = self.state_mut(pin)
        else {
            return;
        };
        *pull_up = enabled;
        gpio.configure(
            pin,
            if enabled {
                PinMode::InputPullup
            } else {
                PinMode::Input
            },
        );
        *previous = read_pin(map, gpio, pin);
    }

    fn set_listening<G: GpioHal>(
        &mut self,
        map: &PinMap,
        target: Target,
        enabled: bool,
        id: RequestId,
        gpio: &G,
    ) -> FirmwareResponse {
        if let Target::Pin(pin) = target
            && let Err(error) = self.initialized(map, pin)
        {
            return error;
        }
        for pin in map
            .pins_for(target)
            .filter(|pin| map.pin(*pin).capabilities.input())
        {
            self.set_listener_pin(map, pin, enabled, id, gpio);
        }
        Response::Ack
    }

    fn set_listener_pin<G: GpioHal>(
        &mut self,
        map: &PinMap,
        pin: PinId,
        enabled: bool,
        id: RequestId,
        gpio: &G,
    ) {
        let PinState::Input {
            listener, previous, ..
        } = self.state_mut(pin)
        else {
            return;
        };
        *listener = enabled.then_some(id);
        if enabled {
            *previous = read_pin(map, gpio, pin);
        }
    }

    fn initialized(&self, map: &PinMap, pin: PinId) -> Result<(), FirmwareResponse> {
        supported(map, pin)?;
        if matches!(self.state(pin), PinState::Unset) {
            return Err(pin_error(map, pin, TargetError::Unset));
        }
        Ok(())
    }

    fn query(&self, pin: PinId, what: Query) -> QueryValue {
        match (self.state(pin), what) {
            (PinState::Unset, _) => QueryValue::Unset,
            (PinState::Input { .. }, Query::Direction) => QueryValue::Direction(Direction::Input),
            (PinState::Output, Query::Direction) => QueryValue::Direction(Direction::Output),
            (PinState::Input { pull_up, .. }, Query::Pullup) => QueryValue::Toggle(pull_up.into()),
            (PinState::Output, Query::Pullup) => QueryValue::Toggle(Toggle::Off),
            (PinState::Input { listener, .. }, Query::Listen) => {
                QueryValue::Toggle(listener.is_some().into())
            }
            (PinState::Output, Query::Listen) => QueryValue::Toggle(Toggle::Off),
        }
    }

    fn reset<G: GpioHal>(&mut self, gpio: &mut G) {
        self.bulk = None;
        self.listener_cursor = 0;
        let map = gpio.pin_map();
        for pin in map.pin_ids() {
            let state = self.state_mut(pin);
            if !matches!(state, PinState::Unset) && map.pin(pin).capabilities.input() {
                gpio.configure(pin, PinMode::Input);
            }
            *state = PinState::Unset;
        }
    }

    fn state(&self, pin: PinId) -> PinState {
        self.pins[pin.index()]
    }

    fn state_mut(&mut self, pin: PinId) -> &mut PinState {
        &mut self.pins[pin.index()]
    }
}

fn target_contains(map: &PinMap, target: Target, pin: PinId) -> bool {
    match target {
        Target::Pin(target) => target == pin,
        Target::Bank(bank) => map.pin(pin).bank == bank,
        Target::All => true,
    }
}

fn supported(map: &PinMap, pin: PinId) -> Result<(), FirmwareResponse> {
    map.pin(pin)
        .capabilities
        .available()
        .then_some(())
        .ok_or_else(|| pin_error(map, pin, TargetError::Unavailable))
}

fn pin_error(map: &PinMap, pin: PinId, reason: TargetError) -> FirmwareResponse {
    Response::Error(ResponseError::Target {
        target: map.pin(pin).token.as_bytes(),
        reason,
    })
}

fn bad_packet() -> FirmwareResponse {
    Response::Error(ResponseError::BadPacket)
}

fn read_pin<G: GpioHal>(map: &PinMap, gpio: &G, pin: PinId) -> Level {
    let info = map.pin(pin);
    level_from_bank(gpio.read_bank(info.bank), info.bit)
}

fn level_from_bank(bits: u32, bit: u8) -> Level {
    if bits & (1u32 << bit) == 0 {
        Level::Low
    } else {
        Level::High
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use core::cell::Cell;

    use super::*;
    use crate::{
        gpio::map::{BankId, Capabilities, PinInfo},
        sam::{SAM_IDENTITY, SAM_PIN_MAP},
    };
    use da_vinci_protocol::Request;

    const BANK_0: BankId = BankId::new(0);
    const BANK_1: BankId = BankId::new(1);

    static SYNTH_BANKS: [&str; 2] = ["PIO0", "PORTX"];
    static SYNTH_PINS: [PinInfo; 4] = [
        PinInfo::new("PIO0_0", Some(1), BANK_0, 0, Capabilities::GPIO),
        PinInfo::new("PIO0_1", Some(2), BANK_0, 1, Capabilities::NONE),
        PinInfo::new("PX07", None, BANK_1, 7, Capabilities::INPUT_PULLUP),
        PinInfo::new("PX08", Some(8), BANK_1, 8, Capabilities::GPIO),
    ];
    static SYNTH_MAP: PinMap = PinMap::new(&SYNTH_BANKS, &SYNTH_PINS);

    struct FakeHal {
        map: &'static PinMap,
        values: [Level; MAX_PINS],
        inputs: [bool; MAX_PINS],
        outputs: [bool; MAX_PINS],
        pull_ups: [bool; MAX_PINS],
        bank_reads: Cell<[u16; MAX_BANKS]>,
    }

    impl FakeHal {
        fn new(map: &'static PinMap) -> Self {
            Self {
                map,
                values: [Level::Low; MAX_PINS],
                inputs: [false; MAX_PINS],
                outputs: [false; MAX_PINS],
                pull_ups: [false; MAX_PINS],
                bank_reads: Cell::new([0; MAX_BANKS]),
            }
        }

        fn reset_reads(&self) {
            self.bank_reads.set([0; MAX_BANKS]);
        }
    }

    impl GpioHal for FakeHal {
        fn pin_map(&self) -> &'static PinMap {
            self.map
        }

        fn configure(&mut self, pin: PinId, mode: PinMode) {
            match mode {
                PinMode::Input => {
                    self.inputs[pin.index()] = true;
                    self.outputs[pin.index()] = false;
                    self.pull_ups[pin.index()] = false;
                }
                PinMode::InputPullup => {
                    self.inputs[pin.index()] = true;
                    self.outputs[pin.index()] = false;
                    self.pull_ups[pin.index()] = true;
                }
                PinMode::Output { initial } => {
                    self.inputs[pin.index()] = false;
                    self.outputs[pin.index()] = true;
                    self.pull_ups[pin.index()] = false;
                    self.values[pin.index()] = initial;
                }
            }
        }

        fn write(&mut self, pin: PinId, level: Level) {
            self.values[pin.index()] = level;
        }

        fn read_bank(&self, bank: BankId) -> u32 {
            let mut reads = self.bank_reads.get();
            reads[bank.index()] += 1;
            self.bank_reads.set(reads);

            self.map.pins_for(Target::Bank(bank)).fold(0, |bits, pin| {
                let info = self.map.pin(pin);
                if self.values[pin.index()] == Level::High {
                    bits | (1u32 << info.bit)
                } else {
                    bits
                }
            })
        }
    }

    fn request(id: u16, body: Request<&'static [u8]>) -> Packet<DecodedRequest<'static>> {
        Packet {
            id: RequestId::new(id).unwrap(),
            body,
        }
    }

    fn firmware() -> Firmware {
        Firmware::new(b"SYNTH GPIO")
    }

    #[test]
    fn pin_map_resolves_native_pin_bank_and_all_without_mcu_branches() {
        assert_eq!(SYNTH_MAP.resolve(b"PIO0"), Some(Target::Bank(BANK_0)));
        assert_eq!(SYNTH_MAP.resolve(b"ALL"), Some(Target::All));
        assert_eq!(
            SYNTH_MAP.resolve(b"PIO0_0"),
            Some(Target::Pin(SYNTH_MAP.pin_id(0)))
        );
        assert_eq!(
            SYNTH_MAP.resolve(b"PX08"),
            Some(Target::Pin(SYNTH_MAP.pin_id(3)))
        );
        assert_eq!(SYNTH_MAP.resolve(b"PA00"), None);
    }

    #[test]
    fn map_stream_uses_synthetic_pin_map_and_terminal_ack() {
        let mut firmware = firmware();
        let mut gpio = FakeHal::new(&SYNTH_MAP);
        let first = firmware.handle(request(10, Request::Map), &mut gpio);
        assert_eq!(
            first,
            Packet {
                id: RequestId::new(10).unwrap(),
                body: Response::MapBank {
                    bank: b"PIO0".as_slice(),
                },
            }
        );
        assert_eq!(
            firmware.poll_bulk(&gpio),
            Some(Packet {
                id: RequestId::new(10).unwrap(),
                body: Response::MapBank {
                    bank: b"PORTX".as_slice(),
                },
            })
        );

        for (index, info) in SYNTH_MAP.pins().iter().enumerate() {
            assert_eq!(
                firmware.poll_bulk(&gpio),
                Some(Packet {
                    id: RequestId::new(10).unwrap(),
                    body: Response::MapPin {
                        target: info.token.as_bytes(),
                        package_pin: info.package_pin,
                        bank: SYNTH_MAP.bank(info.bank).as_bytes(),
                        bit: info.bit,
                        capabilities: info.capabilities,
                    },
                }),
                "pin record {index}"
            );
        }
        assert_eq!(firmware.poll_bulk(&gpio).unwrap().body, Response::Ack);
        assert!(firmware.poll_bulk(&gpio).is_none());
    }

    #[test]
    fn help_streams_canonical_commands_then_terminal_ack() {
        let mut firmware = firmware();
        let mut gpio = FakeHal::new(&SYNTH_MAP);
        let id = RequestId::new(12).unwrap();

        assert_eq!(
            firmware.handle(request(11, Request::Version), &mut gpio),
            Packet {
                id: RequestId::new(11).unwrap(),
                body: Response::Version {
                    version: PROTOCOL_VERSION,
                },
            }
        );

        let mut packet = Some(firmware.handle(request(12, Request::Help), &mut gpio));
        for &command in Command::ALL {
            assert_eq!(
                packet,
                Some(Packet {
                    id,
                    body: Response::Help { command },
                })
            );
            packet = firmware.poll_bulk(&gpio);
        }
        assert_eq!(
            packet,
            Some(Packet {
                id,
                body: Response::Ack,
            })
        );
        assert!(firmware.poll_bulk(&gpio).is_none());
    }

    #[test]
    fn real_sam_map_stream_records_all_metadata_within_frame_limit() {
        use da_vinci_protocol::{Frame, Message};

        let mut firmware = Firmware::new(SAM_IDENTITY);
        let mut gpio = FakeHal::new(&SAM_PIN_MAP);
        let mut packet = Some(firmware.handle(request(11, Request::Map), &mut gpio));
        let mut banks = 0;
        let mut pins = 0;
        let mut unavailable = 0;

        while let Some(current) = packet {
            Frame::try_from(Message {
                route: b"SAM".as_slice(),
                packet: current,
            })
            .expect("every SAM MAP record must fit");
            match current.body {
                Response::MapBank { .. } => banks += 1,
                Response::MapPin { capabilities, .. } => {
                    pins += 1;
                    unavailable += usize::from(!capabilities.available());
                }
                Response::Ack => break,
                _ => panic!("MAP stream emitted unrelated response"),
            }
            packet = firmware.poll_bulk(&gpio);
        }

        assert_eq!(banks, SAM_PIN_MAP.banks().len());
        assert_eq!(pins, SAM_PIN_MAP.pins().len());
        assert_eq!(unavailable, 6);
    }

    #[test]
    fn direction_pullup_read_and_identity_use_map_metadata() {
        let mut firmware = firmware();
        let mut gpio = FakeHal::new(&SYNTH_MAP);

        assert_eq!(
            firmware.handle(request(1, Request::Status), &mut gpio).body,
            Response::Status {
                identity: b"SYNTH GPIO".as_slice(),
            }
        );
        assert_eq!(
            firmware
                .handle(request(2, Request::Get { target: b"PIO0_0" }), &mut gpio)
                .body,
            pin_error(&SYNTH_MAP, SYNTH_MAP.pin_id(0), TargetError::Unset)
        );

        firmware.handle(
            request(
                3,
                Request::Direction {
                    target: b"PIO0_0",
                    direction: Direction::Input,
                },
            ),
            &mut gpio,
        );
        firmware.handle(
            request(
                4,
                Request::Pullup {
                    target: b"PIO0_0",
                    state: Toggle::On,
                },
            ),
            &mut gpio,
        );
        assert!(gpio.inputs[0]);
        assert!(gpio.pull_ups[0]);
        gpio.values[0] = Level::High;
        assert_eq!(
            firmware
                .handle(request(5, Request::Get { target: b"PIO0_0" }), &mut gpio)
                .body,
            Response::Value {
                target: b"PIO0_0".as_slice(),
                level: Level::High,
            }
        );
    }

    #[test]
    fn pullup_does_not_reconfigure_output_pins() {
        let mut firmware = firmware();
        let mut gpio = FakeHal::new(&SYNTH_MAP);
        firmware.handle(
            request(
                1,
                Request::Direction {
                    target: b"PIO0_0",
                    direction: Direction::Output,
                },
            ),
            &mut gpio,
        );
        assert_eq!(gpio.values[0], Level::Low);

        firmware.handle(
            request(
                2,
                Request::Pullup {
                    target: b"PIO0_0",
                    state: Toggle::On,
                },
            ),
            &mut gpio,
        );

        assert!(!gpio.pull_ups[0]);
        assert_eq!(
            firmware.query(SYNTH_MAP.pin_id(0), Query::Pullup),
            QueryValue::Toggle(Toggle::Off)
        );
    }

    #[test]
    fn capability_and_unknown_target_errors_are_local_to_the_map() {
        let mut firmware = firmware();
        let mut gpio = FakeHal::new(&SYNTH_MAP);

        assert_eq!(
            firmware
                .handle(
                    request(
                        1,
                        Request::Direction {
                            target: b"PIO0_1",
                            direction: Direction::Input,
                        },
                    ),
                    &mut gpio,
                )
                .body,
            pin_error(&SYNTH_MAP, SYNTH_MAP.pin_id(1), TargetError::Unavailable)
        );
        assert_eq!(
            firmware
                .handle(request(2, Request::Get { target: b"PA00" }), &mut gpio)
                .body,
            bad_packet()
        );
        assert_eq!(
            firmware
                .handle(
                    request(
                        3,
                        Request::Direction {
                            target: b"PX07",
                            direction: Direction::Output,
                        },
                    ),
                    &mut gpio,
                )
                .body,
            pin_error(&SYNTH_MAP, SYNTH_MAP.pin_id(2), TargetError::Unavailable)
        );
    }

    #[test]
    fn grouped_mutations_follow_active_map_and_skip_unavailable_pins() {
        let mut firmware = firmware();
        let mut gpio = FakeHal::new(&SYNTH_MAP);
        firmware.handle(
            request(
                1,
                Request::Direction {
                    target: b"ALL",
                    direction: Direction::Output,
                },
            ),
            &mut gpio,
        );
        firmware.handle(
            request(
                2,
                Request::Set {
                    target: b"ALL",
                    level: Level::High,
                },
            ),
            &mut gpio,
        );

        assert!(gpio.outputs[0]);
        assert!(!gpio.outputs[1]);
        assert!(!gpio.outputs[2]);
        assert!(gpio.outputs[3]);
        assert_eq!(gpio.values[0], Level::High);
        assert_eq!(gpio.values[3], Level::High);
    }

    #[test]
    fn grouped_get_and_query_stream_until_terminal_ack() {
        let mut firmware = firmware();
        let mut gpio = FakeHal::new(&SYNTH_MAP);
        for target in [b"PIO0_0".as_slice(), b"PX08"] {
            firmware.handle(
                request(
                    1,
                    Request::Direction {
                        target,
                        direction: Direction::Input,
                    },
                ),
                &mut gpio,
            );
        }
        gpio.values[3] = Level::High;

        assert_eq!(
            firmware.handle(request(20, Request::Get { target: b"ALL" }), &mut gpio),
            Packet {
                id: RequestId::new(20).unwrap(),
                body: Response::Value {
                    target: b"PIO0_0".as_slice(),
                    level: Level::Low,
                },
            }
        );
        assert_eq!(
            firmware.poll_bulk(&gpio),
            Some(Packet {
                id: RequestId::new(20).unwrap(),
                body: Response::Value {
                    target: b"PX08".as_slice(),
                    level: Level::High,
                },
            })
        );
        assert_eq!(
            firmware.poll_bulk(&gpio),
            Some(Packet {
                id: RequestId::new(20).unwrap(),
                body: Response::Ack,
            })
        );

        let first = firmware.handle(
            request(
                21,
                Request::Query {
                    target: b"PORTX",
                    what: Query::Direction,
                },
            ),
            &mut gpio,
        );
        assert_eq!(
            first.body,
            Response::State {
                target: b"PX07".as_slice(),
                what: Query::Direction,
                value: QueryValue::Unset,
            }
        );
        assert_eq!(
            firmware.poll_bulk(&gpio),
            Some(Packet {
                id: RequestId::new(21).unwrap(),
                body: Response::State {
                    target: b"PX08".as_slice(),
                    what: Query::Direction,
                    value: QueryValue::Direction(Direction::Input),
                },
            })
        );
        assert_eq!(firmware.poll_bulk(&gpio).unwrap().body, Response::Ack);
    }

    #[test]
    fn listeners_keep_request_ids_read_each_bank_once_and_rotate_fairly() {
        let mut firmware = firmware();
        let mut gpio = FakeHal::new(&SYNTH_MAP);
        for (id, target) in [(10, b"PIO0_0".as_slice()), (11, b"PX08")] {
            firmware.handle(
                request(
                    id,
                    Request::Direction {
                        target,
                        direction: Direction::Input,
                    },
                ),
                &mut gpio,
            );
            firmware.handle(
                request(
                    id + 100,
                    Request::Listen {
                        target,
                        state: Toggle::On,
                    },
                ),
                &mut gpio,
            );
        }

        gpio.reset_reads();
        assert_eq!(firmware.poll_listener(&gpio), None);
        let reads = gpio.bank_reads.get();
        assert_eq!(reads[BANK_0.index()], 1);
        assert_eq!(reads[BANK_1.index()], 1);

        gpio.values[0] = Level::High;
        gpio.values[3] = Level::High;
        assert_eq!(
            firmware.poll_listener(&gpio),
            Some(Packet {
                id: RequestId::new(110).unwrap(),
                body: Response::Value {
                    target: b"PIO0_0".as_slice(),
                    level: Level::High,
                },
            })
        );
        gpio.values[0] = Level::Low;
        assert_eq!(
            firmware.poll_listener(&gpio),
            Some(Packet {
                id: RequestId::new(111).unwrap(),
                body: Response::Value {
                    target: b"PX08".as_slice(),
                    level: Level::High,
                },
            })
        );
    }

    #[test]
    fn changing_listened_input_to_output_drops_input_only_state() {
        let mut firmware = firmware();
        let mut gpio = FakeHal::new(&SYNTH_MAP);
        let pin = SYNTH_MAP.pin_id(0);

        firmware.handle(
            request(
                1,
                Request::Direction {
                    target: b"PIO0_0",
                    direction: Direction::Input,
                },
            ),
            &mut gpio,
        );
        firmware.handle(
            request(
                110,
                Request::Listen {
                    target: b"PIO0_0",
                    state: Toggle::On,
                },
            ),
            &mut gpio,
        );
        assert_eq!(
            firmware.query(pin, Query::Listen),
            QueryValue::Toggle(Toggle::On)
        );

        firmware.handle(
            request(
                2,
                Request::Direction {
                    target: b"PIO0_0",
                    direction: Direction::Output,
                },
            ),
            &mut gpio,
        );

        assert_eq!(
            firmware.query(pin, Query::Direction),
            QueryValue::Direction(Direction::Output)
        );
        assert_eq!(
            firmware.query(pin, Query::Pullup),
            QueryValue::Toggle(Toggle::Off)
        );
        assert_eq!(
            firmware.query(pin, Query::Listen),
            QueryValue::Toggle(Toggle::Off)
        );
        gpio.values[pin.index()] = Level::High;
        assert_eq!(firmware.poll_listener(&gpio), None);
    }

    #[test]
    fn listener_on_output_is_acknowledged_without_becoming_active() {
        let mut firmware = firmware();
        let mut gpio = FakeHal::new(&SYNTH_MAP);
        let pin = SYNTH_MAP.pin_id(0);

        firmware.handle(
            request(
                1,
                Request::Direction {
                    target: b"PIO0_0",
                    direction: Direction::Output,
                },
            ),
            &mut gpio,
        );
        assert_eq!(
            firmware
                .handle(
                    request(
                        120,
                        Request::Listen {
                            target: b"PIO0_0",
                            state: Toggle::On,
                        },
                    ),
                    &mut gpio,
                )
                .body,
            Response::Ack
        );
        assert_eq!(
            firmware.query(pin, Query::Listen),
            QueryValue::Toggle(Toggle::Off)
        );
        gpio.values[pin.index()] = Level::High;
        assert_eq!(firmware.poll_listener(&gpio), None);
    }

    #[test]
    fn bye_releases_initialized_pins_and_listener_state() {
        let mut firmware = firmware();
        let mut gpio = FakeHal::new(&SYNTH_MAP);
        firmware.handle(
            request(
                1,
                Request::Direction {
                    target: b"PIO0_0",
                    direction: Direction::Output,
                },
            ),
            &mut gpio,
        );
        firmware.handle(
            request(
                2,
                Request::Listen {
                    target: b"PIO0_0",
                    state: Toggle::On,
                },
            ),
            &mut gpio,
        );
        assert_eq!(
            firmware.handle(request(3, Request::Bye), &mut gpio).body,
            Response::Bye
        );
        assert!(gpio.inputs[0]);
        assert_eq!(firmware.poll_listener(&gpio), None);
        assert_eq!(
            firmware
                .handle(request(4, Request::Get { target: b"PIO0_0" }), &mut gpio)
                .body,
            pin_error(&SYNTH_MAP, SYNTH_MAP.pin_id(0), TargetError::Unset)
        );
    }
}
