use da_vinci_protocol::{
    DecodeError, Frame, Message, Packet, RawMessage, RequestId, Response, ResponseError,
};

const ROUTE_QUEUE_CAPACITY: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameError {
    WouldBlock,
    Down,
}

pub trait FrameLink {
    fn try_send(&mut self, frame: &Frame) -> Result<(), FrameError>;
    fn try_receive(&mut self) -> Result<Option<Frame>, FrameError>;
}

pub struct Route<'a> {
    next_hop: &'static [u8],
    destinations: &'static [&'static [u8]],
    link: &'a mut dyn FrameLink,
    queue: FrameQueue,
    down: bool,
}

impl<'a> Route<'a> {
    pub fn new(
        next_hop: &'static [u8],
        destinations: &'static [&'static [u8]],
        link: &'a mut dyn FrameLink,
    ) -> Self {
        Self {
            next_hop,
            destinations,
            link,
            queue: FrameQueue::new(),
            down: false,
        }
    }

    fn reaches(&self, destination: &[u8]) -> bool {
        self.destinations.contains(&destination)
    }

    fn forward(&mut self, id: RequestId, frame: Frame) -> Result<(), RouteFailure> {
        if self.down {
            return Err(RouteFailure::Down);
        }
        if self.queue.is_empty() {
            match self.link.try_send(&frame) {
                Ok(()) => return Ok(()),
                Err(FrameError::WouldBlock) => {}
                Err(FrameError::Down) => {
                    self.down = true;
                    return Err(RouteFailure::Down);
                }
            }
        }
        self.queue
            .push(id, frame)
            .then_some(())
            .ok_or(RouteFailure::Busy)
    }

    fn poll_send(&mut self) -> Option<Packet<Response<&'static [u8], &'static [u8]>>> {
        let queued = *self.queue.front()?;
        if self.down {
            self.queue.pop();
            return Some(Packet {
                id: queued.id,
                body: Response::Error(ResponseError::RouteDown {
                    next_hop: self.next_hop,
                }),
            });
        }
        match self.link.try_send(&queued.frame) {
            Ok(()) => {
                self.queue.pop();
                None
            }
            Err(FrameError::WouldBlock) => None,
            Err(FrameError::Down) => {
                self.down = true;
                self.queue.pop();
                Some(Packet {
                    id: queued.id,
                    body: Response::Error(ResponseError::RouteDown {
                        next_hop: self.next_hop,
                    }),
                })
            }
        }
    }

    fn poll_receive(&mut self) -> Option<Frame> {
        if self.down {
            return None;
        }
        match self.link.try_receive() {
            Ok(Some(frame)) => Some(frame),
            Ok(None) | Err(FrameError::WouldBlock) => None,
            Err(FrameError::Down) => {
                self.down = true;
                None
            }
        }
    }
}

#[derive(Clone, Copy)]
enum RouteFailure {
    Busy,
    Down,
}

pub struct Router<'a, const N: usize> {
    local_route: &'static [u8],
    routes: [Route<'a>; N],
    send_cursor: usize,
    receive_cursor: usize,
}

type DispatchResult<'frame, T> = Result<Option<Packet<Response<T, &'frame [u8]>>>, DecodeError>;

impl<'a, const N: usize> Router<'a, N> {
    pub const fn new(local_route: &'static [u8], routes: [Route<'a>; N]) -> Self {
        Self {
            local_route,
            routes,
            send_cursor: 0,
            receive_cursor: 0,
        }
    }

    pub const fn local_route(&self) -> &'static [u8] {
        self.local_route
    }

    pub fn dispatch<'frame, F, T>(
        &mut self,
        frame: &'frame Frame,
        local: F,
    ) -> DispatchResult<'frame, T>
    where
        F: FnOnce(RawMessage<'frame>) -> Packet<Response<T, &'frame [u8]>>,
    {
        let envelope = RawMessage::try_from(frame)?;
        if envelope.route == self.local_route {
            return Ok(Some(local(envelope)));
        }
        let Message { route, packet } = envelope;

        let Some(route_link) = self
            .routes
            .iter_mut()
            .find(|route_link| route_link.reaches(route))
        else {
            return Ok(Some(Packet {
                id: packet.id,
                body: Response::Error(ResponseError::NoRoute { destination: route }),
            }));
        };

        Ok(match route_link.forward(packet.id, *frame) {
            Ok(()) => None,
            Err(RouteFailure::Busy) => Some(Packet {
                id: packet.id,
                body: Response::Error(ResponseError::RouteBusy {
                    next_hop: route_link.next_hop,
                }),
            }),
            Err(RouteFailure::Down) => Some(Packet {
                id: packet.id,
                body: Response::Error(ResponseError::RouteDown {
                    next_hop: route_link.next_hop,
                }),
            }),
        })
    }

    pub fn poll_routes(&mut self) -> Option<Packet<Response<&'static [u8], &'static [u8]>>> {
        if N == 0 {
            return None;
        }
        for _ in 0..N {
            let index = self.send_cursor;
            self.send_cursor = (self.send_cursor + 1) % N;
            let route = &mut self.routes[index];
            if let Some(response) = route.poll_send() {
                return Some(response);
            }
        }
        None
    }

    pub fn poll_upstream(&mut self) -> Option<Frame> {
        if N == 0 {
            return None;
        }
        for _ in 0..N {
            let index = self.receive_cursor;
            self.receive_cursor = (self.receive_cursor + 1) % N;
            if let Some(frame) = self.routes[index].poll_receive() {
                return Some(frame);
            }
        }
        None
    }
}

#[derive(Clone, Copy)]
struct QueuedFrame {
    id: RequestId,
    frame: Frame,
}

struct FrameQueue {
    frames: [Option<QueuedFrame>; ROUTE_QUEUE_CAPACITY],
    head: usize,
    len: usize,
}

impl FrameQueue {
    const fn new() -> Self {
        Self {
            frames: [None; ROUTE_QUEUE_CAPACITY],
            head: 0,
            len: 0,
        }
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn push(&mut self, id: RequestId, frame: Frame) -> bool {
        if self.len == ROUTE_QUEUE_CAPACITY {
            return false;
        }
        let index = (self.head + self.len) % ROUTE_QUEUE_CAPACITY;
        self.frames[index] = Some(QueuedFrame { id, frame });
        self.len += 1;
        true
    }

    fn front(&self) -> Option<&QueuedFrame> {
        self.frames[self.head].as_ref()
    }

    fn pop(&mut self) {
        debug_assert!(self.len != 0);
        self.frames[self.head] = None;
        self.head = (self.head + 1) % ROUTE_QUEUE_CAPACITY;
        self.len -= 1;
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use std::{
        cell::{Cell, RefCell},
        collections::VecDeque,
        rc::Rc,
        vec::Vec,
    };

    #[derive(Clone, Copy)]
    enum SendMode {
        Ready,
        Blocked,
        Down,
    }

    struct FakeLink {
        mode: Rc<Cell<SendMode>>,
        sent: Rc<RefCell<Vec<Vec<u8>>>>,
        incoming: Rc<RefCell<VecDeque<Vec<u8>>>>,
    }

    impl FakeLink {
        fn new(mode: SendMode) -> (Self, LinkControl) {
            let mode = Rc::new(Cell::new(mode));
            let sent = Rc::new(RefCell::new(Vec::new()));
            let incoming = Rc::new(RefCell::new(VecDeque::new()));
            (
                Self {
                    mode: mode.clone(),
                    sent: sent.clone(),
                    incoming: incoming.clone(),
                },
                LinkControl {
                    mode,
                    sent,
                    incoming,
                },
            )
        }
    }

    impl FrameLink for FakeLink {
        fn try_send(&mut self, frame: &Frame) -> Result<(), FrameError> {
            match self.mode.get() {
                SendMode::Ready => {
                    self.sent.borrow_mut().push(frame.as_ref().to_vec());
                    Ok(())
                }
                SendMode::Blocked => Err(FrameError::WouldBlock),
                SendMode::Down => Err(FrameError::Down),
            }
        }

        fn try_receive(&mut self) -> Result<Option<Frame>, FrameError> {
            let Some(bytes) = self.incoming.borrow_mut().pop_front() else {
                return Ok(None);
            };
            Ok(Some(Frame::try_from(bytes.as_slice()).unwrap()))
        }
    }

    struct LinkControl {
        mode: Rc<Cell<SendMode>>,
        sent: Rc<RefCell<Vec<Vec<u8>>>>,
        incoming: Rc<RefCell<VecDeque<Vec<u8>>>>,
    }

    fn frame(bytes: &[u8]) -> Frame {
        Frame::try_from(bytes).unwrap()
    }

    fn request_id(raw: u16) -> RequestId {
        RequestId::new(raw).unwrap()
    }

    fn dispatch<'a, const N: usize>(
        router: &mut Router<'_, N>,
        frame: &'a Frame,
    ) -> Option<Packet<Response<&'a [u8], &'a [u8]>>> {
        router
            .dispatch(frame, |message| Packet {
                id: message.packet.id,
                body: Response::Hello,
            })
            .unwrap()
    }

    #[test]
    fn dispatches_local_body_without_route_knowledge_in_handler() {
        let mut router = Router::new(b"SAM", []);
        let frame = frame(b"010 SAM HAI\n");
        let response = router
            .dispatch(&frame, |message| {
                assert_eq!(message.packet.id, request_id(10));
                assert_eq!(message.packet.body, b"HAI");
                Packet {
                    id: message.packet.id,
                    body: Response::<&[u8], &[u8]>::Hello,
                }
            })
            .unwrap()
            .unwrap();
        assert_eq!(
            response,
            Packet {
                id: request_id(10),
                body: Response::Hello
            }
        );
    }

    #[test]
    fn missing_routes_return_local_no_route_errors() {
        let mut router = Router::new(b"SAM", []);
        for bytes in [b"011 LPC HAI\n".as_slice(), b"012 ABC HAI\n"] {
            let frame = frame(bytes);
            let envelope = RawMessage::try_from(&frame).unwrap();
            assert_eq!(
                dispatch(&mut router, &frame),
                Some(Packet {
                    id: envelope.packet.id,
                    body: Response::Error(ResponseError::NoRoute {
                        destination: envelope.route,
                    }),
                })
            );
        }
    }

    #[test]
    fn route_can_reach_multiple_destinations_without_rewriting_frames() {
        let (mut link, control) = FakeLink::new(SendMode::Ready);
        let mut router = Router::new(b"SAM", [Route::new(b"LPC", &[b"LPC", b"ABC"], &mut link)]);
        let lpc = frame(b"021 LPC HAI\n");
        let abc = frame(b"022 ABC GET PIO2_3 OK?\n");
        assert_eq!(dispatch(&mut router, &lpc), None);
        assert_eq!(dispatch(&mut router, &abc), None);
        assert_eq!(
            control.sent.borrow().as_slice(),
            [lpc.as_ref(), abc.as_ref()]
        );
    }

    #[test]
    fn bounded_queue_reports_busy_and_stalled_route_does_not_block_another() {
        let (mut stalled, stalled_control) = FakeLink::new(SendMode::Blocked);
        let (mut ready, ready_control) = FakeLink::new(SendMode::Ready);
        let mut router = Router::new(
            b"SAM",
            [
                Route::new(b"LPC", &[b"LPC"], &mut stalled),
                Route::new(b"XYZ", &[b"XYZ"], &mut ready),
            ],
        );
        for id in 31..=32 {
            let bytes = std::format!("{id:03} LPC HAI\n").into_bytes();
            let frame = frame(&bytes);
            assert_eq!(dispatch(&mut router, &frame), None);
        }
        let busy = frame(b"033 LPC HAI\n");
        assert_eq!(
            dispatch(&mut router, &busy),
            Some(Packet {
                id: request_id(33),
                body: Response::Error(ResponseError::RouteBusy {
                    next_hop: b"LPC".as_slice(),
                }),
            })
        );

        let other = frame(b"034 XYZ HAI\n");
        assert_eq!(dispatch(&mut router, &other), None);
        assert_eq!(ready_control.sent.borrow().as_slice(), [other.as_ref()]);

        stalled_control.mode.set(SendMode::Ready);
        assert_eq!(router.poll_routes(), None);
        assert_eq!(router.poll_routes(), None);
        assert_eq!(stalled_control.sent.borrow().len(), 2);
    }

    #[test]
    fn hard_link_failure_is_route_down_not_busy() {
        let (mut link, _) = FakeLink::new(SendMode::Down);
        let mut router = Router::new(b"SAM", [Route::new(b"LPC", &[b"LPC"], &mut link)]);
        let frame = frame(b"041 LPC HAI\n");
        assert_eq!(
            dispatch(&mut router, &frame),
            Some(Packet {
                id: request_id(41),
                body: Response::Error(ResponseError::RouteDown {
                    next_hop: b"LPC".as_slice(),
                }),
            })
        );
    }

    #[test]
    fn queued_frame_that_discovers_link_failure_returns_its_original_request_id() {
        let (mut link, control) = FakeLink::new(SendMode::Blocked);
        let mut router = Router::new(b"SAM", [Route::new(b"LPC", &[b"LPC"], &mut link)]);
        let frame = frame(b"051 LPC HAI\n");
        assert_eq!(dispatch(&mut router, &frame), None);
        control.mode.set(SendMode::Down);
        assert_eq!(
            router.poll_routes(),
            Some(Packet {
                id: request_id(51),
                body: Response::Error(ResponseError::RouteDown {
                    next_hop: b"LPC".as_slice(),
                }),
            })
        );
    }

    #[test]
    fn downstream_responses_and_events_return_upstream_unchanged() {
        let (mut link, control) = FakeLink::new(SendMode::Ready);
        let mut router = Router::new(b"SAM", [Route::new(b"LPC", &[b"LPC"], &mut link)]);
        let response = b"061 LPC HII :3\n".to_vec();
        let event = b"062 LPC HYG PIO2_3 HIGH :3\n".to_vec();
        control
            .incoming
            .borrow_mut()
            .extend([response.clone(), event.clone()]);
        assert_eq!(router.poll_upstream().unwrap().as_ref(), response);
        assert_eq!(router.poll_upstream().unwrap().as_ref(), event);
    }

    #[test]
    fn two_hop_chain_preserves_request_and_response_frames() {
        let (mut sam_to_lpc, sam_control) = FakeLink::new(SendMode::Ready);
        let (mut lpc_to_abc, lpc_control) = FakeLink::new(SendMode::Ready);
        let request = frame(b"071 ABC HAI\n");

        {
            let mut sam = Router::new(
                b"SAM",
                [Route::new(b"LPC", &[b"LPC", b"ABC"], &mut sam_to_lpc)],
            );
            assert_eq!(dispatch(&mut sam, &request), None);
        }
        let forwarded = sam_control.sent.borrow()[0].clone();
        let forwarded = frame(&forwarded);
        {
            let mut lpc = Router::new(b"LPC", [Route::new(b"ABC", &[b"ABC"], &mut lpc_to_abc)]);
            assert_eq!(dispatch(&mut lpc, &forwarded), None);
        }
        assert_eq!(lpc_control.sent.borrow().as_slice(), [request.as_ref()]);

        let response = b"071 ABC HII <3\n".to_vec();
        sam_control
            .incoming
            .borrow_mut()
            .push_back(response.clone());
        let mut sam = Router::new(
            b"SAM",
            [Route::new(b"LPC", &[b"LPC", b"ABC"], &mut sam_to_lpc)],
        );
        assert_eq!(sam.poll_upstream().unwrap().as_ref(), response);
    }
}
