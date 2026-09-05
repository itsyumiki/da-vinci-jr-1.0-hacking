use da_vinci_protocol::{
    DecodeError, DecodeErrorKind, DecodedRequest, Frame, Message, Packet, RawMessage, Response,
    ResponseError,
};

use crate::{
    gpio::{Firmware, GpioHal},
    router::{Route, Router},
    transport::{ByteError, FramedTransport, NonBlockingBytes},
};

const SOURCE_COUNT: usize = 5;
const ROUTE_INPUT: usize = 0;
const ROUTE_PROGRESS: usize = 1;
const BULK: usize = 2;
const REQUEST: usize = 3;
const LISTENER: usize = 4;

pub struct Node<'route, const N: usize> {
    firmware: Firmware,
    router: Router<'route, N>,
    upstream: FramedTransport,
    source_cursor: usize,
}

impl<'route, const N: usize> Node<'route, N> {
    pub const fn new(
        identity: &'static [u8],
        local_route: &'static [u8],
        routes: [Route<'route>; N],
    ) -> Self {
        Self {
            firmware: Firmware::new(identity),
            router: Router::new(local_route, routes),
            upstream: FramedTransport::new(),
            source_cursor: REQUEST,
        }
    }

    pub fn poll<B: NonBlockingBytes, G: GpioHal>(
        &mut self,
        upstream: &mut B,
        gpio: &mut G,
    ) -> Result<(), ByteError> {
        self.upstream.poll(upstream)?;
        if self.upstream.tx_idle() {
            self.schedule(gpio);
        }
        self.upstream.poll(upstream)
    }

    fn schedule<G: GpioHal>(&mut self, gpio: &mut G) {
        for _ in 0..SOURCE_COUNT {
            let source = self.source_cursor;
            self.source_cursor = (self.source_cursor + 1) % SOURCE_COUNT;
            let produced = match source {
                ROUTE_INPUT => self.route_input(),
                ROUTE_PROGRESS => self.route_progress(),
                BULK => self.bulk(gpio),
                REQUEST => self.request(gpio),
                LISTENER => self.listener(gpio),
                _ => unreachable!(),
            };
            if produced || !self.upstream.tx_idle() {
                return;
            }
        }
    }

    fn route_input(&mut self) -> bool {
        let Some(frame) = self.router.poll_upstream() else {
            return false;
        };
        self.upstream
            .enqueue(frame.as_ref())
            .expect("routed frame fits idle upstream transport");
        true
    }

    fn route_progress(&mut self) -> bool {
        let Some(packet) = self.router.poll_routes() else {
            return false;
        };
        queue_response(&mut self.upstream, self.router.local_route(), packet);
        true
    }

    fn bulk<G: GpioHal>(&mut self, gpio: &G) -> bool {
        let Some(packet) = self.firmware.poll_bulk(gpio) else {
            return false;
        };
        queue_response(&mut self.upstream, self.router.local_route(), packet);
        true
    }

    fn request<G: GpioHal>(&mut self, gpio: &mut G) -> bool {
        let Ok(Some(frame)) = self.upstream.next_frame() else {
            return false;
        };
        match RawMessage::try_from(&frame) {
            Ok(envelope) => {
                let firmware = &mut self.firmware;
                let response = self.router.dispatch(frame.as_ref(), envelope, |raw| {
                    Message::<&[u8], DecodedRequest<'_>>::try_from(raw)
                        .map(|message| firmware.handle(message.packet, gpio))
                        .unwrap_or_else(|error| {
                            decode_error_response::<&[u8]>(error)
                                .expect("local command decode errors keep their ID")
                        })
                });
                if let Some(response) = response {
                    queue_response(&mut self.upstream, self.router.local_route(), response);
                }
            }
            Err(error) => {
                if let Some(response) = decode_error_response::<&[u8]>(error) {
                    queue_response(&mut self.upstream, self.router.local_route(), response);
                }
            }
        }
        true
    }

    fn listener<G: GpioHal>(&mut self, gpio: &G) -> bool {
        let Some(packet) = self.firmware.poll_listener(gpio) else {
            return false;
        };
        queue_response(&mut self.upstream, self.router.local_route(), packet);
        true
    }
}

fn queue_response<T: AsRef<[u8]>, D: AsRef<[u8]>>(
    transport: &mut FramedTransport,
    source: &[u8],
    packet: Packet<Response<T, D>>,
) {
    let frame = Frame::try_from(Message {
        route: source,
        packet,
    })
    .expect("protocol response always fits fixed packet buffer");
    transport
        .enqueue(frame.as_ref())
        .expect("response queued only while upstream transport is idle");
}

fn decode_error_response<T>(error: DecodeError) -> Option<Packet<Response<T, &'static [u8]>>> {
    let id = error.id?;
    let body = match error.kind {
        DecodeErrorKind::Malformed => Response::Error(ResponseError::BadPacket),
        DecodeErrorKind::UnknownCommand => Response::Unknown,
    };
    Some(Packet { id, body })
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::{cell::RefCell, collections::VecDeque, rc::Rc, vec::Vec};

    use da_vinci_protocol::{Command, Level};

    use super::*;
    use crate::{
        BankId, PinId, PinMap, PinMode,
        gpio::map::{BankInfo, Capabilities, PinInfo},
        router::{FrameError, FrameLink},
    };

    const BANK: BankId = BankId::new(0);
    static BANKS: [BankInfo; 1] = [BankInfo::new("PIO2")];
    static PINS: [PinInfo; 1] = [PinInfo::new(
        "PIO2_3",
        Some(38),
        BANK,
        3,
        Capabilities::GPIO,
    )];
    static MAP: PinMap = PinMap::new(&BANKS, &PINS);

    struct FakeGpio {
        bank: u32,
    }

    impl GpioHal for FakeGpio {
        fn pin_map(&self) -> &'static PinMap {
            &MAP
        }

        fn configure(&mut self, _: PinId, mode: PinMode) {
            if let PinMode::Output { initial } = mode {
                self.bank = match initial {
                    Level::Low => 0,
                    Level::High => 1 << 3,
                };
            }
        }

        fn write(&mut self, _: PinId, level: Level) {
            self.bank = match level {
                Level::Low => 0,
                Level::High => 1 << 3,
            };
        }

        fn read_bank(&self, _: BankId) -> u32 {
            self.bank
        }
    }

    struct FakeBytes {
        input: Vec<u8>,
        read: usize,
        output: Vec<u8>,
    }

    impl FakeBytes {
        fn new(input: &[u8]) -> Self {
            Self {
                input: input.to_vec(),
                read: 0,
                output: Vec::new(),
            }
        }
    }

    impl NonBlockingBytes for FakeBytes {
        fn try_read(&mut self, out: &mut [u8]) -> Result<usize, ByteError> {
            if self.read == self.input.len() {
                return Err(ByteError::WouldBlock);
            }
            let count = out.len().min(self.input.len() - self.read);
            out[..count].copy_from_slice(&self.input[self.read..self.read + count]);
            self.read += count;
            Ok(count)
        }

        fn try_write(&mut self, bytes: &[u8]) -> Result<usize, ByteError> {
            self.output.extend_from_slice(bytes);
            Ok(bytes.len())
        }
    }

    #[test]
    fn local_commands_and_missing_routes_share_the_node_loop() {
        let mut bytes =
            FakeBytes::new(b"101 LPC HAI\n102 LPC HRU\n103 LPC DIR PIO2_3 IN OK?\n104 XYZ HAI\n");
        let mut gpio = FakeGpio { bank: 0 };
        let mut node = Node::new(b"LPC1115 GPIO", b"LPC", []);

        for _ in 0..16 {
            node.poll(&mut bytes, &mut gpio).unwrap();
        }

        assert_eq!(
            bytes.output,
            b"101 LPC HII :3\n102 LPC IAM LPC1115 GPIO :3\n103 LPC OKA :3\n104 LPC UMM NO_ROUTE XYZ :3\n"
        );
    }

    #[test]
    fn protocol_diagnostics_work_through_lpc_node() {
        let mut bytes = FakeBytes::new(b"301 LPC VER\n302 LPC HLP\n");
        let mut gpio = FakeGpio { bank: 0 };
        let mut node = Node::new(b"LPC1115 GPIO", b"LPC", []);

        for _ in 0..64 {
            node.poll(&mut bytes, &mut gpio).unwrap();
        }

        let mut expected = b"301 LPC VER 1 :3\n".to_vec();
        for command in Command::ALL {
            expected.extend_from_slice(b"302 LPC HLP ");
            expected.extend_from_slice(command.as_ref());
            expected.extend_from_slice(b" :3\n");
        }
        expected.extend_from_slice(b"302 LPC OKA :3\n");
        assert_eq!(bytes.output, expected);
    }

    struct FakeFrameLink {
        sent: Rc<RefCell<Vec<Vec<u8>>>>,
        incoming: Rc<RefCell<VecDeque<Vec<u8>>>>,
    }

    impl FrameLink for FakeFrameLink {
        fn try_send(&mut self, frame: &[u8]) -> Result<(), FrameError> {
            self.sent.borrow_mut().push(frame.to_vec());
            Ok(())
        }

        fn try_receive(&mut self) -> Result<Option<Frame>, FrameError> {
            let Some(bytes) = self.incoming.borrow_mut().pop_front() else {
                return Ok(None);
            };
            Ok(Some(Frame::try_from(bytes.as_slice()).unwrap()))
        }
    }

    #[test]
    fn routed_frames_are_opaque_and_preserve_ids_both_directions() {
        let sent = Rc::new(RefCell::new(Vec::new()));
        let incoming = Rc::new(RefCell::new(VecDeque::new()));
        let mut link = FakeFrameLink {
            sent: Rc::clone(&sent),
            incoming: Rc::clone(&incoming),
        };
        let route = Route::new(b"LPC", &[b"LPC"], &mut link);
        let mut node = Node::new(b"SAM4E8E GPIO", b"SAM", [route]);
        let mut bytes = FakeBytes::new(b"200 LPC HAI\n201 SAM HAI\n");
        let mut gpio = FakeGpio { bank: 0 };

        for _ in 0..12 {
            node.poll(&mut bytes, &mut gpio).unwrap();
        }

        assert_eq!(&*sent.borrow(), &[b"200 LPC HAI\n".to_vec()]);
        assert_eq!(bytes.output, b"201 SAM HII <3\n");

        incoming
            .borrow_mut()
            .push_back(b"200 LPC HII :3\n".to_vec());
        incoming
            .borrow_mut()
            .push_back(b"230 LPC HYG PIO2_3 HIGH :3\n".to_vec());
        for _ in 0..12 {
            node.poll(&mut bytes, &mut gpio).unwrap();
        }

        assert_eq!(
            bytes.output,
            b"201 SAM HII <3\n200 LPC HII :3\n230 LPC HYG PIO2_3 HIGH :3\n"
        );
    }
}
