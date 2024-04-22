#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(clippy::needless_doctest_main)]
//! craftping provides a `ping` function to send Server List Ping requests to a Minecraft server.
//!
//! # Feature flags
//!
//! - `sync` (default): Enables synchronous, blocking [`ping`](crate::sync::ping) function.
//! - `async-tokio`: Enables asynchronous, `tokio`-based [`ping`](crate::tokio::ping) function.
//! - `async-futures`: Enables asynchronous, `futures`-based [`ping`](crate::futures::ping) function.
//!
//! # Examples
//!
//! ```no_run
//! use craftping::sync::ping;
//! use std::net::TcpStream;
//!
//! fn main() {
//!     let hostname = "my.server.com";
//!     let port = 25565;
//!     let mut stream = TcpStream::connect((hostname, port)).unwrap();
//!     let response = ping(&mut stream, hostname, port).unwrap();
//!     println!("Players online: {}", response.online_players);
//! }
//! ```

use std::{
    fmt::Display,
    io::{Read, Write},
};

mod entity;
#[cfg(feature = "async-futures")]
#[cfg_attr(docsrs, doc(cfg(feature = "async-futures")))]
pub mod futures;
#[cfg(feature = "sync")]
#[cfg_attr(docsrs, doc(cfg(feature = "sync")))]
pub mod sync;
#[cfg(feature = "async-tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "async-tokio")))]
pub mod tokio;

pub use entity::*;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
/// The ping error type.
pub enum Error {
    /// Returned when I/O (especially networking) failed.
    Io(std::io::Error),
    /// Returned when the response cannot be recognized.
    UnsupportedProtocol,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(io) => io.fmt(f),
            Self::UnsupportedProtocol => write!(f, "unsupported protocol"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

/// The ping result type.
pub type Result<T> = std::result::Result<T, Error>;

fn build_latest_request(hostname: &str, port: u16) -> Result<Vec<u8>> {
    // buffer for the 1st packet's data part
    let mut buffer = vec![
        0x00, // 1st packet id: 0 for handshake as VarInt
        0xff, 0xff, 0xff, 0xff,
        0x0f, // protocol version: -1 (determining what version to use) as VarInt
    ];
    // Some server implementations require hostname and port to be properly set (Notchian does not)
    write_varint(&mut buffer, hostname.len() as i32); // length of hostname as VarInt
    buffer.extend_from_slice(hostname.as_bytes());
    buffer.extend_from_slice(&[
        (port >> 8) as u8,
        (port & 0b1111_1111) as u8, // server port as unsigned short
        0x01,                       // next state: 1 (status) as VarInt
    ]);
    // buffer for the 1st and 2nd packet
    let mut full_buffer = vec![];
    write_varint(&mut full_buffer, buffer.len() as i32); // length of 1st packet id + data as VarInt
    full_buffer.append(&mut buffer);
    full_buffer.extend_from_slice(&[
        1,    // length of 2nd packet id + data as VarInt
        0x00, // 2nd packet id: 0 for request as VarInt
    ]);
    Ok(full_buffer)
}

fn decode_latest_response(buffer: &[u8]) -> Result<RawLatest> {
    serde_json::from_slice(buffer).map_err(|_| Error::UnsupportedProtocol)
}

const LEGACY_REQUEST: [u8; 35] = [
    0xfe, // 1st packet id: 0xfe for server list ping
    0x01, // payload: always 1
    0xfa, // 2nd packet id: 0xfa for plugin message
    0x00, 0x0b, // length of following string: always 11 as short,
    0x00, 0x4d, 0x00, 0x43, 0x00, 0x7c, 0x00, 0x50, 0x00, 0x69, 0x00, 0x6e, 0x00, 0x67, 0x00, 0x48,
    0x00, 0x6f, 0x00, 0x73, 0x00, 0x74,
    // MC|PingHost as UTF16-BE
    7,    // length of the rest of the data: 7 + length of hostname
    0x4a, // protocol version: 0x4a for the last version
    0x00, 0x00, // length of hostname: 0 as short
    0x00, 0x00, 0x00, 0x00, // port: 0 as int
];

fn decode_legacy(buffer: &[u8]) -> Result<String> {
    if buffer.len() <= 3 || buffer[0] != 0xff {
        return Err(Error::UnsupportedProtocol);
    }
    let utf16be: Vec<u16> = buffer[3..]
        .chunks_exact(2)
        .map(|chunk| ((chunk[0] as u16) << 8) | chunk[1] as u16)
        .collect();
    String::from_utf16(&utf16be).map_err(|_| Error::UnsupportedProtocol)
}

fn parse_legacy(s: &str, raw: Vec<u8>) -> Result<Response> {
    let mut fields = s.split('\0');
    let magic = fields.next().map(|s| s == "\u{00a7}\u{0031}");
    let protocol = fields.next().and_then(|s| s.parse().ok());
    let version = fields.next();
    let motd = fields.next();
    let players = fields.next().and_then(|s| s.parse().ok());
    let max_players = fields.next().and_then(|s| s.parse().ok());
    match (magic, protocol, version, motd, players, max_players) {
        (
            Some(true),
            Some(protocol),
            Some(version),
            Some(motd),
            Some(players),
            Some(max_players),
        ) => Ok(Response {
            protocol,
            enforces_secure_chat: None,
            previews_chat: None,
            version: version.to_string(),
            description: Chat {
                text: motd.to_string(),
                ..Default::default()
            },
            online_players: players,
            max_players,
            favicon: None,
            forge_data: None,
            mod_info: None,
            sample: None,
            raw,
        }),
        _ => Err(Error::UnsupportedProtocol),
    }
}

// used in read_varint implemenetation
const LAST_SEVEN_BITS: i32 = 0b0111_1111;
const NEXT_BYTE_EXISTS: u8 = 0b1000_0000;

// bit mask to remove remaining 7 MSB's after right shift
const SEVEN_BITS_SHIFT_MASK: i32 = 0x01_ff_ff_ff;

fn write_varint(sink: &mut Vec<u8>, mut value: i32) {
    loop {
        let mut temp = (value & LAST_SEVEN_BITS) as u8;
        // i32 right shift is arithmetic shift (preserves MSB)
        value >>= 7;
        value &= SEVEN_BITS_SHIFT_MASK;
        if value != 0 {
            temp |= NEXT_BYTE_EXISTS;
        }
        sink.push(temp);
        if value == 0 {
            break;
        }
    }
}
