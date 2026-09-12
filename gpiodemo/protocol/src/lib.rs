#![no_std]

mod codec;
mod command;
mod framing;
mod message;

pub use codec::{DecodeError, DecodeErrorKind, EncodeError};
pub use command::{
    Command, DecodedRequest, DecodedResponse, Direction, Level, PROTOCOL_VERSION, ParseTokenError,
    PinCapabilities, Query, QueryValue, Request, Response, ResponseError, TargetError, Toggle,
};
pub use framing::{Frame, LineBuffer, LineError, MAX_PACKET_LEN};
pub use message::{Message, Packet, RawMessage, RequestId};

#[cfg(test)]
mod tests;
