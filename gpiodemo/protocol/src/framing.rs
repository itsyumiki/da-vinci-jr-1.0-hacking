pub const MAX_PACKET_LEN: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameTooLong;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frame {
    bytes: [u8; MAX_PACKET_LEN],
    len: usize,
}

impl TryFrom<&[u8]> for Frame {
    type Error = FrameTooLong;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() > MAX_PACKET_LEN {
            return Err(FrameTooLong);
        }
        let mut frame = Self {
            bytes: [0; MAX_PACKET_LEN],
            len: bytes.len(),
        };
        frame.bytes[..bytes.len()].copy_from_slice(bytes);
        Ok(frame)
    }
}

impl Frame {
    pub(crate) const fn from_parts(bytes: [u8; MAX_PACKET_LEN], len: usize) -> Self {
        Self { bytes, len }
    }
}

impl AsRef<[u8]> for Frame {
    fn as_ref(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineError {
    TooLong,
}

pub struct LineBuffer {
    bytes: [u8; MAX_PACKET_LEN],
    len: usize,
    discarding: bool,
}

impl Default for LineBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl LineBuffer {
    pub const fn new() -> Self {
        Self {
            bytes: [0; MAX_PACKET_LEN],
            len: 0,
            discarding: false,
        }
    }

    pub fn clear(&mut self) {
        self.len = 0;
        self.discarding = false;
    }

    pub fn push(&mut self, byte: u8) -> Result<Option<&[u8]>, LineError> {
        if byte == b'\r' {
            return Ok(None);
        }
        if byte == b'\n' {
            let line = (!self.discarding && self.len != 0).then_some(&self.bytes[..self.len]);
            self.len = 0;
            self.discarding = false;
            return Ok(line);
        }
        if self.discarding {
            return Ok(None);
        }
        if self.len + 1 >= self.bytes.len() {
            self.len = 0;
            self.discarding = true;
            return Err(LineError::TooLong);
        }
        self.bytes[self.len] = byte;
        self.len += 1;
        Ok(None)
    }
}
