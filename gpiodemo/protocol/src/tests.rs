extern crate std;
use super::*;

fn id(raw: u16) -> RequestId {
    RequestId::new(raw).unwrap()
}

fn encoded_request(id: RequestId, body: Request<&'static [u8]>) -> [u8; MAX_PACKET_LEN] {
    let mut out = [0u8; MAX_PACKET_LEN];
    let len = encode_request(Packet { id, body }, b"SAM", &mut out).unwrap();
    assert_eq!(decoded_request(&out[..len]), Ok(Packet { id, body }));
    out
}

fn decoded_request(line: &[u8]) -> Result<Packet<DecodedRequest<'_>>, DecodeError> {
    let envelope = decode_message(line)?;
    decode_request(envelope.packet)
}

fn decoded_response(line: &[u8]) -> Result<Packet<DecodedResponse<'_>>, DecodeError> {
    let envelope = decode_message(line)?;
    decode_response(envelope.packet)
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
fn routed_envelopes_borrow_route_and_opaque_body() {
    let request = b"001 SAM HAI";
    let envelope = decode_message(request).unwrap();
    assert_eq!(
        envelope,
        Message {
            route: b"SAM",
            packet: Packet {
                id: id(1),
                body: b"HAI",
            },
        }
    );
    assert_eq!(envelope.route.as_ptr(), request[4..].as_ptr());
    assert_eq!(envelope.packet.body.as_ptr(), request[8..].as_ptr());

    assert_eq!(
        decode_message(b"002 LPC GET PIO2_3 OK?"),
        Ok(Message {
            route: b"LPC",
            packet: Packet {
                id: id(2),
                body: b"GET PIO2_3 OK?",
            },
        })
    );
    assert_eq!(
        decode_message(b"003 ABC WAT opaque body"),
        Ok(Message {
            route: b"ABC",
            packet: Packet {
                id: id(3),
                body: b"WAT opaque body",
            },
        })
    );
    assert_eq!(
        decode_message(b"002 LPC HYG PIO2_3 HIGH <3"),
        Ok(Message {
            route: b"LPC",
            packet: Packet {
                id: id(2),
                body: b"HYG PIO2_3 HIGH <3",
            },
        })
    );
}

#[test]
fn routed_envelope_encoding_validates_route_tokens_and_preserves_ids() {
    let mut out = [0; MAX_PACKET_LEN];
    let request = Message {
        route: b"ABC",
        packet: Packet {
            id: id(999),
            body: b"HAI",
        },
    };
    let len = encode_message(request, &mut out).unwrap();
    assert_eq!(&out[..len], b"999 ABC HAI\n");
    assert_eq!(decode_message(&out[..len]), Ok(request));

    let response = Message {
        route: b"SAM",
        packet: Packet {
            id: id(7),
            body: b"HII <3",
        },
    };
    let len = encode_message(response, &mut out).unwrap();
    assert_eq!(&out[..len], b"007 SAM HII <3\n");
    assert_eq!(decode_message(&out[..len]), Ok(response));

    for route in [b"".as_slice(), b"BAD ROUTE", b"BAD\nROUTE", b"\x01"] {
        assert_eq!(
            encode_message(
                Message {
                    route,
                    packet: Packet {
                        id: id(1),
                        body: b"HAI",
                    },
                },
                &mut out,
            ),
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
                enabled: false,
            },
            "001 SAM PLL PIOB OFF OK?\n",
        ),
        (
            Request::Listen {
                target: b"PIOE".as_slice(),
                enabled: true,
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
                enabled: true,
            },
            "001 SAM PLL ALL ON OK?\n",
        ),
        (
            Request::Listen {
                target: b"ALL".as_slice(),
                enabled: true,
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
    ];

    for (body, expected) in cases {
        let out = encoded_request(id(1), body);
        assert_eq!(&out[..expected.len()], expected.as_bytes());
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
                value: QueryValue::Enabled(true),
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
        let mut out = [0u8; MAX_PACKET_LEN];
        let len = encode_response(packet, b"SAM", &mut out).unwrap();
        assert_eq!(&out[..len], expected.as_bytes());
        assert_eq!(decoded_response(&out[..len]), Ok(packet));
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
    let mut out = [0; MAX_PACKET_LEN];
    let len = encode_request(request, b"LPC", &mut out).unwrap();
    assert_eq!(&out[..len], b"021 LPC SET PIO2_3 HIGH OK?\n");
    assert_eq!(decoded_request(&out[..len]), Ok(request));

    let response = Packet {
        id: id(22),
        body: Response::Value {
            target: b"PIO2_3".as_slice(),
            level: Level::Low,
        },
    };
    let len = encode_response(response, b"LPC", &mut out).unwrap();
    assert_eq!(&out[..len], b"022 LPC HYG PIO2_3 LOW <3\n");
    assert_eq!(decoded_response(&out[..len]), Ok(response));

    let status = Packet {
        id: id(23),
        body: Response::<&[u8], &[u8]>::Status {
            identity: b"LPC1115 GPIO",
        },
    };
    let len = encode_response(status, b"LPC", &mut out).unwrap();
    assert_eq!(&out[..len], b"023 LPC IAM LPC1115 GPIO <3\n");
    assert_eq!(decoded_response(&out[..len]), Ok(status));
}
