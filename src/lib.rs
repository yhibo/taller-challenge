//! Reads one message that arrived from the network.
//!
//! Format (numbers are big-endian, most significant byte first):
//!   magic    2 bytes   "GW"
//!   type     1 byte    1 = Hello, 2 = Data (there are no other types)
//!   length   4 bytes   payload length (u32), at most 65,536
//!   payload  `length` bytes
//!       Hello: exactly 12 bytes: session id (u64), then keepalive in seconds (u32)
//!       Data:  any bytes
//!
//! At each numbered point, line A is active and lines B and C are turned off with `//`.
//! Keep the best line and delete the other two. Exactly one line must stay at each point.

#[derive(Debug, PartialEq, Eq)]
pub enum Message {
    Hello {
        session_id: u64,
        keepalive_secs: u32,
    },
    Data(Vec<u8>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    TooShort,
    BadMagic,
    TooLarge(u32),
    UnknownType(u8),
    BadHelloLength(usize),
}

pub fn parse(buf: &[u8]) -> Result<Message, ParseError> {
    // 1. Read the 7-byte header.
    let header: &[u8] = buf.get(..7).ok_or(ParseError::TooShort)?; // 1B

    // 2. Check the magic.
    if header[0..2] != *b"GW" {
        return Err(ParseError::BadMagic);
    } // 2C

    let msg_type = header[2];
    let len = u32::from_be_bytes([header[3], header[4], header[5], header[6]]);

    // 3. Check the payload length.
    if len > 65_536 {
        return Err(ParseError::TooLarge(len));
    } // 3B

    // 4. Take the payload.
    let payload = buf.get(7..7 + len as usize).ok_or(ParseError::TooShort)?; // 4C

    // 5. Decide by message type.
    match msg_type {
        1 => hello(payload),
        2 => Ok(Message::Data(payload.to_vec())),
        _ => Err(ParseError::UnknownType(msg_type)),
    } // 5C
}

fn hello(payload: &[u8]) -> Result<Message, ParseError> {
    let p: &[u8; 12] = payload
        .try_into()
        .map_err(|_| ParseError::BadHelloLength(payload.len()))?;

    // 6. Read the session id.
    let session_id = u64::from_be_bytes(p[0..8].try_into().unwrap()); // 6A

    Ok(Message::Hello {
        session_id,
        keepalive_secs: u32::from_be_bytes(p[8..12].try_into().unwrap()),
    })
}
