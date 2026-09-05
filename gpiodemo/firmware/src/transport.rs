use crate::router::{FrameError, FrameLink};
use da_vinci_protocol::{Frame, LineBuffer, LineError, MAX_PACKET_LEN};

const RX_CAPACITY: usize = MAX_PACKET_LEN * 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ByteError {
    WouldBlock,
    Down,
}

pub trait NonBlockingBytes {
    fn try_read(&mut self, out: &mut [u8]) -> Result<usize, ByteError>;
    fn try_write(&mut self, bytes: &[u8]) -> Result<usize, ByteError>;
}
pub struct FramedLink<B> {
    bytes: B,
    transport: FramedTransport,
}

impl<B> FramedLink<B> {
    pub const fn new(bytes: B) -> Self {
        Self {
            bytes,
            transport: FramedTransport::new(),
        }
    }
}

impl<B: NonBlockingBytes> FrameLink for FramedLink<B> {
    fn try_send(&mut self, frame: &[u8]) -> Result<(), FrameError> {
        if frame.len() > MAX_PACKET_LEN {
            return Err(FrameError::InvalidFrame);
        }
        self.transport
            .poll(&mut self.bytes)
            .map_err(FrameError::from)?;
        self.transport.enqueue(frame).map_err(|error| match error {
            QueueError::Busy => FrameError::WouldBlock,
            QueueError::TooLong => FrameError::InvalidFrame,
        })?;
        self.transport
            .poll(&mut self.bytes)
            .map_err(FrameError::from)
    }

    fn try_receive(&mut self) -> Result<Option<Frame>, FrameError> {
        self.transport
            .poll(&mut self.bytes)
            .map_err(FrameError::from)?;
        self.transport
            .next_frame()
            .map_err(|LineError::TooLong| FrameError::InvalidFrame)
    }
}

impl From<ByteError> for FrameError {
    fn from(error: ByteError) -> Self {
        match error {
            ByteError::WouldBlock => Self::WouldBlock,
            ByteError::Down => Self::Down,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueueError {
    Busy,
    TooLong,
}

pub struct FramedTransport {
    rx: RingBuffer,
    line: LineBuffer,
    tx: [u8; MAX_PACKET_LEN],
    tx_len: usize,
    tx_offset: usize,
}

impl Default for FramedTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl FramedTransport {
    pub const fn new() -> Self {
        Self {
            rx: RingBuffer::new(),
            line: LineBuffer::new(),
            tx: [0; MAX_PACKET_LEN],
            tx_len: 0,
            tx_offset: 0,
        }
    }

    pub fn poll<B: NonBlockingBytes>(&mut self, bytes: &mut B) -> Result<(), ByteError> {
        self.flush(bytes)?;
        self.read(bytes)
    }

    pub fn tx_idle(&self) -> bool {
        self.tx_len == 0
    }

    pub fn enqueue(&mut self, frame: &[u8]) -> Result<(), QueueError> {
        if !self.tx_idle() {
            return Err(QueueError::Busy);
        }
        if frame.len() > MAX_PACKET_LEN {
            return Err(QueueError::TooLong);
        }
        self.tx[..frame.len()].copy_from_slice(frame);
        self.tx_len = frame.len();
        self.tx_offset = 0;
        Ok(())
    }

    pub fn next_frame(&mut self) -> Result<Option<Frame>, LineError> {
        while let Some(byte) = self.rx.pop() {
            match self.line.push(byte) {
                Ok(Some(line)) => {
                    let mut bytes = [0; MAX_PACKET_LEN];
                    bytes[..line.len()].copy_from_slice(line);
                    bytes[line.len()] = b'\n';
                    return Ok(Some(
                        Frame::try_from(&bytes[..line.len() + 1])
                            .expect("line buffer enforces protocol frame capacity"),
                    ));
                }
                Ok(None) => {}
                Err(error) => return Err(error),
            }
        }
        Ok(None)
    }

    fn read<B: NonBlockingBytes>(&mut self, bytes: &mut B) -> Result<(), ByteError> {
        let mut input = [0; MAX_PACKET_LEN];
        let capacity = self.rx.free().min(input.len());
        if capacity == 0 {
            return Ok(());
        }
        match bytes.try_read(&mut input[..capacity]) {
            Ok(0) | Err(ByteError::WouldBlock) => Ok(()),
            Ok(count) => {
                debug_assert!(count <= capacity);
                self.rx.extend(&input[..count]);
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    fn flush<B: NonBlockingBytes>(&mut self, bytes: &mut B) -> Result<(), ByteError> {
        if self.tx_idle() {
            return Ok(());
        }
        match bytes.try_write(&self.tx[self.tx_offset..self.tx_len]) {
            Ok(0) | Err(ByteError::WouldBlock) => Ok(()),
            Ok(written) => {
                debug_assert!(self.tx_offset + written <= self.tx_len);
                self.tx_offset += written;
                if self.tx_offset == self.tx_len {
                    self.tx_len = 0;
                    self.tx_offset = 0;
                }
                Ok(())
            }
            Err(error) => Err(error),
        }
    }
}

struct RingBuffer {
    bytes: [u8; RX_CAPACITY],
    head: usize,
    len: usize,
}

impl RingBuffer {
    const fn new() -> Self {
        Self {
            bytes: [0; RX_CAPACITY],
            head: 0,
            len: 0,
        }
    }

    fn free(&self) -> usize {
        RX_CAPACITY - self.len
    }

    fn extend(&mut self, input: &[u8]) {
        debug_assert!(input.len() <= self.free());
        let tail = (self.head + self.len) % RX_CAPACITY;
        let first = input.len().min(RX_CAPACITY - tail);
        self.bytes[tail..tail + first].copy_from_slice(&input[..first]);
        self.bytes[..input.len() - first].copy_from_slice(&input[first..]);
        self.len += input.len();
    }

    fn pop(&mut self) -> Option<u8> {
        if self.len == 0 {
            return None;
        }
        let byte = self.bytes[self.head];
        self.head = (self.head + 1) % RX_CAPACITY;
        self.len -= 1;
        Some(byte)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use std::{collections::VecDeque, vec::Vec};

    struct FakeBytes {
        reads: VecDeque<Result<Vec<u8>, ByteError>>,
        writes: Vec<u8>,
        write_limit: usize,
        write_result: Option<ByteError>,
    }

    impl FakeBytes {
        fn new(reads: impl IntoIterator<Item = Result<Vec<u8>, ByteError>>) -> Self {
            Self {
                reads: reads.into_iter().collect(),
                writes: Vec::new(),
                write_limit: usize::MAX,
                write_result: None,
            }
        }
    }

    impl NonBlockingBytes for FakeBytes {
        fn try_read(&mut self, out: &mut [u8]) -> Result<usize, ByteError> {
            let Some(next) = self.reads.pop_front() else {
                return Err(ByteError::WouldBlock);
            };
            let input = next?;
            let len = input.len().min(out.len());
            out[..len].copy_from_slice(&input[..len]);
            if len != input.len() {
                self.reads.push_front(Ok(input[len..].to_vec()));
            }
            Ok(len)
        }

        fn try_write(&mut self, bytes: &[u8]) -> Result<usize, ByteError> {
            if let Some(error) = self.write_result.take() {
                return Err(error);
            }
            let len = bytes.len().min(self.write_limit);
            self.writes.extend_from_slice(&bytes[..len]);
            Ok(len)
        }
    }

    fn frame(transport: &mut FramedTransport) -> Result<Option<Vec<u8>>, LineError> {
        transport
            .next_frame()
            .map(|frame| frame.map(|frame| frame.as_ref().to_vec()))
    }

    #[test]
    fn split_and_batched_frames_stay_in_order() {
        let mut bytes =
            FakeBytes::new([Ok(b"001 SAM HA".to_vec()), Ok(b"I\n002 SAM HRU\n".to_vec())]);
        let mut transport = FramedTransport::new();

        transport.poll(&mut bytes).unwrap();
        assert_eq!(frame(&mut transport), Ok(None));
        transport.poll(&mut bytes).unwrap();
        assert_eq!(frame(&mut transport), Ok(Some(b"001 SAM HAI\n".to_vec())));
        assert_eq!(frame(&mut transport), Ok(Some(b"002 SAM HRU\n".to_vec())));
    }

    #[test]
    fn partial_writes_resume_and_would_block_is_not_failure() {
        let mut bytes = FakeBytes::new([]);
        bytes.write_limit = 3;
        let mut transport = FramedTransport::new();
        transport.enqueue(b"001 SAM HII <3\n").unwrap();

        transport.poll(&mut bytes).unwrap();
        bytes.write_result = Some(ByteError::WouldBlock);
        transport.poll(&mut bytes).unwrap();
        assert!(!transport.tx_idle());
        while !transport.tx_idle() {
            transport.poll(&mut bytes).unwrap();
        }
        assert_eq!(bytes.writes, b"001 SAM HII <3\n");
    }

    #[test]
    fn zero_progress_is_backpressure_and_hard_failure_is_distinct() {
        let mut bytes = FakeBytes::new([Ok(Vec::new()), Err(ByteError::WouldBlock)]);
        bytes.write_limit = 0;
        let mut transport = FramedTransport::new();
        transport.enqueue(b"frame\n").unwrap();

        assert_eq!(transport.poll(&mut bytes), Ok(()));
        assert_eq!(transport.poll(&mut bytes), Ok(()));
        bytes.write_result = Some(ByteError::Down);
        assert_eq!(transport.poll(&mut bytes), Err(ByteError::Down));
    }

    #[test]
    fn oversized_line_is_discarded_and_next_frame_recovers() {
        let mut oversized = std::vec![b'x'; MAX_PACKET_LEN + 8];
        oversized.extend_from_slice(b"\n007 SAM HAI\n");
        let mut bytes = FakeBytes::new([Ok(oversized)]);
        let mut transport = FramedTransport::new();

        for _ in 0..3 {
            transport.poll(&mut bytes).unwrap();
        }
        assert_eq!(frame(&mut transport), Err(LineError::TooLong));
        assert_eq!(frame(&mut transport), Ok(Some(b"007 SAM HAI\n".to_vec())));
    }

    #[test]
    fn transmit_queue_is_fixed_to_one_complete_frame() {
        let mut transport = FramedTransport::new();
        assert_eq!(
            transport.enqueue(&[b'x'; MAX_PACKET_LEN + 1]),
            Err(QueueError::TooLong)
        );
        assert_eq!(transport.enqueue(b"one\n"), Ok(()));
        assert_eq!(transport.enqueue(b"two\n"), Err(QueueError::Busy));
    }
}

#[cfg(test)]
mod framed_link_tests {
    extern crate std;

    use std::{collections::VecDeque, vec::Vec};

    use super::*;

    struct Bytes {
        reads: VecDeque<Vec<u8>>,
        writes: Vec<u8>,
        write_limit: usize,
    }

    impl NonBlockingBytes for Bytes {
        fn try_read(&mut self, out: &mut [u8]) -> Result<usize, ByteError> {
            let Some(mut input) = self.reads.pop_front() else {
                return Err(ByteError::WouldBlock);
            };
            let len = input.len().min(out.len());
            out[..len].copy_from_slice(&input[..len]);
            if len != input.len() {
                input.drain(..len);
                self.reads.push_front(input);
            }
            Ok(len)
        }

        fn try_write(&mut self, bytes: &[u8]) -> Result<usize, ByteError> {
            let len = bytes.len().min(self.write_limit);
            self.writes.extend_from_slice(&bytes[..len]);
            Ok(len)
        }
    }

    #[test]
    fn framed_link_owns_partial_uart_tx_and_line_rx() {
        let bytes = Bytes {
            reads: [b"200 LPC HII ".to_vec(), b":3\n".to_vec()].into(),
            writes: Vec::new(),
            write_limit: 3,
        };
        let mut link = FramedLink::new(bytes);

        assert_eq!(link.try_send(b"200 LPC HAI\n"), Ok(()));

        let mut response = None;
        for _ in 0..8 {
            if let Some(frame) = link.try_receive().unwrap() {
                response = Some(frame.as_ref().to_vec());
            }
        }

        assert_eq!(link.bytes.writes, b"200 LPC HAI\n");
        assert_eq!(response, Some(b"200 LPC HII :3\n".to_vec()));
    }

    #[test]
    fn framed_link_rejects_oversized_send_without_marking_bytes_down() {
        let bytes = Bytes {
            reads: VecDeque::new(),
            writes: Vec::new(),
            write_limit: usize::MAX,
        };
        let mut link = FramedLink::new(bytes);

        assert_eq!(
            link.try_send(&[b'x'; MAX_PACKET_LEN + 1]),
            Err(FrameError::InvalidFrame)
        );
        assert_eq!(link.try_send(b"200 LPC HAI\n"), Ok(()));
        assert_eq!(link.bytes.writes, b"200 LPC HAI\n");
    }
}
