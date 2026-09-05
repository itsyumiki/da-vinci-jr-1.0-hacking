#![no_std]

mod codec;
mod command;
mod framing;
mod message;

pub use codec::{
    DecodeError, DecodeErrorKind, EncodeError, decode_message, decode_request, decode_response,
    encode_message, encode_request, encode_response,
};
pub use command::{
    DecodedRequest, DecodedResponse, Direction, Level, ParseTokenError, PinCapabilities, Query,
    QueryValue, Request, Response, ResponseError, TargetError,
};
pub use framing::{LineBuffer, LineError, MAX_PACKET_LEN};
pub use message::{Message, Packet, RequestId};

#[cfg(test)]
mod tests;
