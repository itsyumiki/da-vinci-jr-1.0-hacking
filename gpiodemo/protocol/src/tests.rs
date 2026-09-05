extern crate std;
use super::*;

fn id(raw: u16) -> RequestId {
    RequestId::new(raw).unwrap()
}

#[test]
fn response_error_maps_generic_fields_at_its_owner() {
    let target = ResponseError::Target {
        target: b"PA00".as_slice(),
        reason: TargetError::Unavailable,
    }
    .try_map(core::str::from_utf8, core::str::from_utf8)
    .unwrap();
    assert_eq!(
        target,
        ResponseError::Target {
            target: "PA00",
            reason: TargetError::Unavailable,
        }
    );

    let route = ResponseError::<&[u8], _>::RouteDown {
        next_hop: b"LPC".as_slice(),
    }
    .try_map(core::str::from_utf8, core::str::from_utf8)
    .unwrap();
    assert_eq!(
        route,
        ResponseError::<&str, _>::RouteDown { next_hop: "LPC" }
    );
}

fn encoded_request(id: RequestId, body: Request<&'static [u8]>) -> Frame {
    let frame = Frame::try_from(Message {
        route: b"SAM".as_slice(),
        packet: Packet { id, body },
    })
    .unwrap();
    assert_eq!(decoded_request(frame.as_ref()), Ok(Packet { id, body }));
    frame
}

fn decoded_request(line: &[u8]) -> Result<Packet<DecodedRequest<'_>>, DecodeError> {
    let raw = RawMessage::try_from(line)?;
    Ok(Message::<&[u8], DecodedRequest<'_>>::try_from(raw)?.packet)
}

fn decoded_response(line: &[u8]) -> Result<Packet<DecodedResponse<'_>>, DecodeError> {
    let raw = RawMessage::try_from(line)?;
    Ok(Message::<&[u8], DecodedResponse<'_>>::try_from(raw)?.packet)
}
#[test]
fn line_buffer_frames_and_recovers_after_overflow() {
    let mut buffer = LineBuffer::new();
    let mut seen = false;
    for &byte in b"\r001 SAM HAI\r\n" {
        if let Some(line) = buffer.push(byte).unwrap() {
            assert_eq!(line, b"001 SAM HAI");
            seen = true;
        }
    }
    assert!(seen);

    for _ in 0..MAX_PACKET_LEN - 1 {
        assert_eq!(buffer.push(b'x'), Ok(None));
    }
    assert_eq!(buffer.push(b'x'), Err(LineError::TooLong));
    assert_eq!(buffer.push(b'x'), Ok(None));
    assert_eq!(buffer.push(b'\n'), Ok(None));

    for &byte in b"008 SAM HII <3\n" {
        if let Some(line) = buffer.push(byte).unwrap() {
            assert_eq!(line, b"008 SAM HII <3");
        }
    }
}

#[test]
fn frame_owns_exact_bounded_bytes() {
    for bytes in [b"".as_slice(), b"001 SAM HAI\n", &[b'x'; MAX_PACKET_LEN]] {
        let frame = Frame::try_from(bytes).unwrap();
        assert_eq!(frame.as_ref(), bytes);
    }
    assert_eq!(
        Frame::try_from(&[b'x'; MAX_PACKET_LEN + 1][..]),
        Err(LineError::TooLong)
    );
}

#[test]
fn routed_envelopes_borrow_route_and_opaque_body() {
    let request = b"001 SAM HAI";
    let envelope = RawMessage::try_from(request.as_slice()).unwrap();
    assert_eq!(
        envelope,
        Message {
            route: b"SAM".as_slice(),
            packet: Packet {
                id: id(1),
                body: b"HAI".as_slice(),
            },
        }
    );
    assert_eq!(envelope.route.as_ptr(), request[4..].as_ptr());
    assert_eq!(envelope.packet.body.as_ptr(), request[8..].as_ptr());

    assert_eq!(
        RawMessage::try_from(b"002 LPC GET PIO2_3 OK?".as_slice()),
        Ok(Message {
            route: b"LPC".as_slice(),
            packet: Packet {
                id: id(2),
                body: b"GET PIO2_3 OK?".as_slice(),
            },
        })
    );
    assert_eq!(
        RawMessage::try_from(b"003 ABC WAT opaque body".as_slice()),
        Ok(Message {
            route: b"ABC".as_slice(),
            packet: Packet {
                id: id(3),
                body: b"WAT opaque body".as_slice(),
            },
        })
    );
    assert_eq!(
        RawMessage::try_from(b"002 LPC HYG PIO2_3 HIGH <3".as_slice()),
        Ok(Message {
            route: b"LPC".as_slice(),
            packet: Packet {
                id: id(2),
                body: b"HYG PIO2_3 HIGH <3".as_slice(),
            },
        })
    );
}

#[test]
fn routed_envelope_encoding_validates_route_tokens_and_preserves_ids() {
    let request = Message {
        route: b"ABC".as_slice(),
        packet: Packet {
            id: id(999),
            body: b"HAI".as_slice(),
        },
    };
    let frame = Frame::try_from(request).unwrap();
    assert_eq!(frame.as_ref(), b"999 ABC HAI\n");
    assert_eq!(RawMessage::try_from(&frame), Ok(request));

    let response = Message {
        route: b"SAM".as_slice(),
        packet: Packet {
            id: id(7),
            body: b"HII <3".as_slice(),
        },
    };
    let frame = Frame::try_from(response).unwrap();
    assert_eq!(frame.as_ref(), b"007 SAM HII <3\n");
    assert_eq!(RawMessage::try_from(&frame), Ok(response));

    for route in [b"".as_slice(), b"BAD ROUTE", b"BAD\nROUTE", b"\x01"] {
        assert_eq!(
            Frame::try_from(Message {
                route,
                packet: Packet {
                    id: id(1),
                    body: b"HAI".as_slice(),
                },
            }),
            Err(EncodeError::InvalidRouteToken)
        );
    }
}

#[test]
fn request_wire_examples_use_symbolic_targets() {
    let cases = [
        (Request::Hello, "001 SAM HAI\n"),
        (Request::Status, "001 SAM HRU\n"),
        (Request::Map, "001 SAM MAP\n"),
        (
            Request::Direction {
                target: b"PA00".as_slice(),
                direction: Direction::Input,
            },
            "001 SAM DIR PA00 IN OK?\n",
        ),
        (
            Request::Direction {
                target: b"PE05".as_slice(),
                direction: Direction::Output,
            },
            "001 SAM DIR PE05 OUT OK?\n",
        ),
        (
            Request::Get {
                target: b"PA05".as_slice(),
            },
            "001 SAM GET PA05 OK?\n",
        ),
        (
            Request::Set {
                target: b"PIOC".as_slice(),
                level: Level::High,
            },
            "001 SAM SET PIOC HIGH OK?\n",
        ),
        (
            Request::Pullup {
                target: b"PIOB".as_slice(),
                state: Toggle::Off,
            },
            "001 SAM PLL PIOB OFF OK?\n",
        ),
        (
            Request::Listen {
                target: b"PIOE".as_slice(),
                state: Toggle::On,
            },
            "001 SAM LSN PIOE ON OK?\n",
        ),
        (
            Request::Query {
                target: b"PC25".as_slice(),
                what: Query::Direction,
            },
            "001 SAM WYD PC25 DIR\n",
        ),
        (
            Request::Direction {
                target: b"ALL".as_slice(),
                direction: Direction::Input,
            },
            "001 SAM DIR ALL IN OK?\n",
        ),
        (
            Request::Get {
                target: b"ALL".as_slice(),
            },
            "001 SAM GET ALL OK?\n",
        ),
        (
            Request::Set {
                target: b"ALL".as_slice(),
                level: Level::High,
            },
            "001 SAM SET ALL HIGH OK?\n",
        ),
        (
            Request::Pullup {
                target: b"ALL".as_slice(),
                state: Toggle::On,
            },
            "001 SAM PLL ALL ON OK?\n",
        ),
        (
            Request::Listen {
                target: b"ALL".as_slice(),
                state: Toggle::On,
            },
            "001 SAM LSN ALL ON OK?\n",
        ),
        (
            Request::Query {
                target: b"ALL".as_slice(),
                what: Query::Listen,
            },
            "001 SAM WYD ALL LSN\n",
        ),
        (Request::Bye, "001 SAM BYE\n"),
        (Request::Version, "001 SAM VER\n"),
        (Request::Help, "001 SAM HLP\n"),
    ];

    for (body, expected) in cases {
        let frame = encoded_request(id(1), body);
        assert_eq!(frame.as_ref(), expected.as_bytes());
    }
}

#[test]
fn request_command_wire_vocabulary_round_trips_and_is_canonical() {
    let expected = [
        (Command::Hello, b"HAI".as_slice()),
        (Command::Status, b"HRU".as_slice()),
        (Command::Map, b"MAP".as_slice()),
        (Command::Direction, b"DIR".as_slice()),
        (Command::Get, b"GET".as_slice()),
        (Command::Set, b"SET".as_slice()),
        (Command::Pullup, b"PLL".as_slice()),
        (Command::Listen, b"LSN".as_slice()),
        (Command::Query, b"WYD".as_slice()),
        (Command::Bye, b"BYE".as_slice()),
        (Command::Version, b"VER".as_slice()),
        (Command::Help, b"HLP".as_slice()),
    ];
    let commands = expected.map(|(command, _)| command);

    assert_eq!(Command::ALL, commands.as_slice());
    for (command, token) in expected {
        assert_eq!(command.as_ref(), token);
        assert_eq!(Command::try_from(token), Ok(command));
    }
    assert_eq!(Command::try_from(b"NOPE".as_slice()), Err(ParseTokenError));
}

#[test]
fn fixed_wire_values_round_trip_from_one_mapping() {
    for (value, token) in [
        (Direction::Input, b"IN".as_slice()),
        (Direction::Output, b"OUT".as_slice()),
    ] {
        assert_eq!(value.as_ref(), token);
        assert_eq!(Direction::try_from(token), Ok(value));
    }
    for (value, token) in [
        (Level::Low, b"LOW".as_slice()),
        (Level::High, b"HIGH".as_slice()),
    ] {
        assert_eq!(value.as_ref(), token);
        assert_eq!(Level::try_from(token), Ok(value));
    }
    for (value, token) in [
        (Query::Direction, b"DIR".as_slice()),
        (Query::Pullup, b"PLL".as_slice()),
        (Query::Listen, b"LSN".as_slice()),
    ] {
        assert_eq!(value.as_ref(), token);
        assert_eq!(Query::try_from(token), Ok(value));
    }
    for (value, token) in [
        (Toggle::Off, b"OFF".as_slice()),
        (Toggle::On, b"ON".as_slice()),
    ] {
        assert_eq!(value.as_ref(), token);
        assert_eq!(Toggle::try_from(token), Ok(value));
    }
    for (value, token) in [
        (TargetError::Unset, b"UNSET".as_slice()),
        (TargetError::Unavailable, b"UNAVAILABLE".as_slice()),
    ] {
        assert_eq!(value.as_ref(), token);
        assert_eq!(TargetError::try_from(token), Ok(value));
    }
}

#[test]
fn response_wire_examples_use_symbolic_pins() {
    let cases = [
        (Response::Hello, "008 SAM HII <3\n"),
        (
            Response::Status {
                identity: b"SAM4E8E GPIO".as_slice(),
            },
            "008 SAM IAM SAM4E8E GPIO <3\n",
        ),
        (
            Response::MapBank {
                bank: b"PIOA".as_slice(),
            },
            "008 SAM MAP BANK PIOA <3\n",
        ),
        (
            Response::MapPin {
                target: b"PA00".as_slice(),
                package_pin: Some(102),
                bank: b"PIOA".as_slice(),
                bit: 0,
                capabilities: PinCapabilities::GPIO,
            },
            "008 SAM MAP PIN PA00 102 PIOA 0 7 <3\n",
        ),
        (
            Response::MapPin {
                target: b"PIO2_3".as_slice(),
                package_pin: None,
                bank: b"PIO2".as_slice(),
                bit: 3,
                capabilities: PinCapabilities::INPUT,
            },
            "008 SAM MAP PIN PIO2_3 - PIO2 3 1 <3\n",
        ),
        (Response::Ack, "008 SAM OKA <3\n"),
        (
            Response::Value {
                target: b"PA00".as_slice(),
                level: Level::High,
            },
            "008 SAM HYG PA00 HIGH <3\n",
        ),
        (
            Response::State {
                target: b"PA00".as_slice(),
                what: Query::Direction,
                value: QueryValue::Direction(Direction::Input),
            },
            "008 SAM HYG PA00 DIR IN <3\n",
        ),
        (
            Response::State {
                target: b"PA00".as_slice(),
                what: Query::Pullup,
                value: QueryValue::Toggle(Toggle::On),
            },
            "008 SAM HYG PA00 PLL ON <3\n",
        ),
        (
            Response::State {
                target: b"PA00".as_slice(),
                what: Query::Listen,
                value: QueryValue::Unset,
            },
            "008 SAM HYG PA00 LSN UNSET <3\n",
        ),
        (
            Response::Version {
                version: PROTOCOL_VERSION,
            },
            "008 SAM VER 1 <3\n",
        ),
        (
            Response::Help {
                command: Command::Hello,
            },
            "008 SAM HLP HAI <3\n",
        ),
        (
            Response::Error(ResponseError::BadPacket),
            "008 SAM UMM BAD_PACKET <3\n",
        ),
        (
            Response::Error(ResponseError::Target {
                target: b"PB08".as_slice(),
                reason: TargetError::Unavailable,
            }),
            "008 SAM UMM PB08 UNAVAILABLE <3\n",
        ),
        (
            Response::Error(ResponseError::Target {
                target: b"PA03".as_slice(),
                reason: TargetError::Unset,
            }),
            "008 SAM UMM PA03 UNSET <3\n",
        ),
        (
            Response::Error(ResponseError::NoRoute {
                destination: b"LPC".as_slice(),
            }),
            "008 SAM UMM NO_ROUTE LPC <3\n",
        ),
        (
            Response::Error(ResponseError::RouteBusy {
                next_hop: b"LPC".as_slice(),
            }),
            "008 SAM UMM ROUTE_BUSY LPC <3\n",
        ),
        (
            Response::Error(ResponseError::RouteDown {
                next_hop: b"LPC".as_slice(),
            }),
            "008 SAM UMM ROUTE_DOWN LPC <3\n",
        ),
        (Response::Unknown, "008 SAM IDK <3\n"),
        (Response::Bye, "008 SAM CYA <3\n"),
    ];

    for (body, expected) in cases {
        let packet = Packet { id: id(8), body };
        let sam = Frame::try_from(Message {
            route: b"SAM".as_slice(),
            packet,
        })
        .unwrap();
        assert_eq!(sam.as_ref(), expected.as_bytes());
        assert_eq!(decoded_response(sam.as_ref()), Ok(packet));

        let lpc = Frame::try_from(Message {
            route: b"LPC".as_slice(),
            packet,
        })
        .unwrap();
        assert!(lpc.as_ref().ends_with(b" :3\n"));
        assert_eq!(decoded_response(lpc.as_ref()), Ok(packet));
    }
}

#[test]
fn map_codec_rejects_bad_numeric_and_capability_fields() {
    for line in [
        b"008 SAM MAP PIN PA00 nope PIOA 0 7 <3".as_slice(),
        b"008 SAM MAP PIN PA00 102 PIOA 999 7 <3",
        b"008 SAM MAP PIN PA00 102 PIOA 0 8 <3",
    ] {
        assert_eq!(
            decoded_response(line),
            Err(DecodeError {
                id: Some(id(8)),
                kind: DecodeErrorKind::Malformed,
            })
        );
    }
}

#[test]
fn malformed_known_and_unknown_requests_are_distinct() {
    assert_eq!(
        decoded_request(b"007 SAM DIR PA00 SIDEWAYS OK?\n"),
        Err(DecodeError {
            id: Some(id(7)),
            kind: DecodeErrorKind::Malformed,
        })
    );
    assert_eq!(
        decoded_request(b"007 SAM WAT PA00\n"),
        Err(DecodeError {
            id: Some(id(7)),
            kind: DecodeErrorKind::UnknownCommand,
        })
    );
    assert_eq!(
        decoded_request(b"nope SAM HAI\n"),
        Err(DecodeError {
            id: None,
            kind: DecodeErrorKind::Malformed,
        })
    );
}

#[test]
fn packet_ids_remain_decimal_but_numeric_gpio_targets_are_rejected() {
    assert_eq!(RequestId::new(0), None);
    assert_eq!(RequestId::new(1), Some(RequestId::FIRST));
    assert_eq!(RequestId::new(999).unwrap().next(), RequestId::FIRST);
    assert_eq!(RequestId::new(1000), None);
    assert_eq!(RequestId::new(1).unwrap().slot(), 0);
    assert_eq!(RequestId::new(999).unwrap().slot(), RequestId::COUNT - 1);
    assert_eq!(
        core::mem::size_of::<Option<RequestId>>(),
        core::mem::size_of::<RequestId>()
    );
    assert!(decoded_request(b"000 SAM HAI").is_err());
    assert_eq!(
        decoded_request(b"9 SAM GET PE05 OK?"),
        Ok(Packet {
            id: id(9),
            body: Request::Get {
                target: b"PE05".as_slice(),
            },
        })
    );
    assert!(decoded_request(b"1000 SAM HAI").is_err());
    assert!(decoded_request(b"001 SAM GET 116 OK?").is_err());
    assert_eq!(
        decoded_request(b"001 SAM GET PE06 OK?"),
        Ok(Packet {
            id: id(1),
            body: Request::Get {
                target: b"PE06".as_slice(),
            },
        })
    );
    assert_eq!(
        decoded_request(b"002 LPC GET PIO2_3 OK?"),
        Ok(Packet {
            id: id(2),
            body: Request::Get {
                target: b"PIO2_3".as_slice(),
            },
        })
    );
}

#[test]
fn typed_codec_round_trips_non_sam_targets_and_identity() {
    let request = Packet {
        id: id(21),
        body: Request::Set {
            target: b"PIO2_3".as_slice(),
            level: Level::High,
        },
    };
    let frame = Frame::try_from(Message {
        route: b"LPC".as_slice(),
        packet: request,
    })
    .unwrap();
    assert_eq!(frame.as_ref(), b"021 LPC SET PIO2_3 HIGH OK?\n");
    assert_eq!(decoded_request(frame.as_ref()), Ok(request));

    let response = Packet {
        id: id(22),
        body: Response::Value {
            target: b"PIO2_3".as_slice(),
            level: Level::Low,
        },
    };
    let frame = Frame::try_from(Message {
        route: b"LPC".as_slice(),
        packet: response,
    })
    .unwrap();
    assert_eq!(frame.as_ref(), b"022 LPC HYG PIO2_3 LOW :3\n");
    assert_eq!(decoded_response(frame.as_ref()), Ok(response));

    let status = Packet {
        id: id(23),
        body: Response::<&[u8], &[u8]>::Status {
            identity: b"LPC1115 GPIO",
        },
    };
    let frame = Frame::try_from(Message {
        route: b"LPC".as_slice(),
        packet: status,
    })
    .unwrap();
    assert_eq!(frame.as_ref(), b"023 LPC IAM LPC1115 GPIO :3\n");
    assert_eq!(decoded_response(frame.as_ref()), Ok(status));
}

#[test]
fn response_terminator_follows_source_and_is_validated_on_decode() {
    let packet = Packet {
        id: id(31),
        body: Response::<&[u8], &[u8]>::Hello,
    };
    let sam = Frame::try_from(Message {
        route: b"SAM".as_slice(),
        packet,
    })
    .unwrap();
    assert_eq!(sam.as_ref(), b"031 SAM HII <3\n");
    assert_eq!(decoded_response(sam.as_ref()), Ok(packet));

    let lpc = Frame::try_from(Message {
        route: b"LPC".as_slice(),
        packet,
    })
    .unwrap();
    assert_eq!(lpc.as_ref(), b"031 LPC HII :3\n");
    assert_eq!(decoded_response(lpc.as_ref()), Ok(packet));

    for line in [b"031 SAM HII :3".as_slice(), b"031 LPC HII <3"] {
        assert_eq!(
            decoded_response(line),
            Err(DecodeError {
                id: Some(id(31)),
                kind: DecodeErrorKind::Malformed,
            })
        );
    }
}

#[test]
fn version_and_help_wire_examples_are_typed_and_source_aware() {
    assert_eq!(PROTOCOL_VERSION, 1);
    assert_eq!(
        decoded_request(b"041 SAM VER"),
        Ok(Packet {
            id: id(41),
            body: Request::Version,
        })
    );
    assert_eq!(
        decoded_request(b"042 SAM HLP"),
        Ok(Packet {
            id: id(42),
            body: Request::Help,
        })
    );
    assert_eq!(
        decoded_response(b"041 SAM VER 1 <3"),
        Ok(Packet {
            id: id(41),
            body: Response::Version { version: 1 },
        })
    );
    assert_eq!(
        decoded_response(b"042 SAM HLP HAI <3"),
        Ok(Packet {
            id: id(42),
            body: Response::Help {
                command: Command::Hello,
            },
        })
    );
    assert_eq!(
        decoded_response(b"043 LPC VER 1 :3"),
        Ok(Packet {
            id: id(43),
            body: Response::Version { version: 1 },
        })
    );
    assert_eq!(
        decoded_response(b"044 LPC HLP HLP :3"),
        Ok(Packet {
            id: id(44),
            body: Response::Help {
                command: Command::Help,
            },
        })
    );

    for source in [b"SAM".as_slice(), b"LPC"] {
        for &command in Command::ALL {
            let packet = Packet {
                id: id(45),
                body: Response::<&[u8], &[u8]>::Help { command },
            };
            let frame = Frame::try_from(Message {
                route: source,
                packet,
            })
            .unwrap();
            assert!(frame.as_ref().len() <= MAX_PACKET_LEN);
            assert_eq!(decoded_response(frame.as_ref()), Ok(packet));
        }
    }
}
