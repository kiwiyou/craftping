use std::{
    convert::TryInto,
    fmt::Display,
    io::{Read, Write},
    net::TcpStream,
};

pub mod entity;
use entity::*;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
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

pub type Result<T> = std::result::Result<T, Error>;

pub fn ping(hostname: &str, port: u16) -> Result<Response> {
    ping_latest(hostname, port).or_else(|_| ping_legacy(hostname, port))
}

fn ping_latest(hostname: &str, port: u16) -> Result<Response> {
    let mut socket = TcpStream::connect((hostname, port))?;
    let mut buffer = vec![
        0x00, // 1st packet id: 0 for handshake as VarInt
        0xff, 0xff, 0xff, 0xff,
        0x0f, // protocol version: -1 (determining what version to use) as VarInt
    ];
    // Some server implementations require hostname and port to be properly set (Notchian does not)
    write_varint(&mut buffer, hostname.len() as i32)?; // length of hostname as VarInt
    buffer.extend_from_slice(hostname.as_bytes());
    buffer.extend_from_slice(&[
        (port >> 8) as u8,
        (port & 0b1111_1111) as u8, // server port as unsigned short
        0x01,                       // next state: 1 (status) as VarInt
    ]);
    write_varint(&mut socket, buffer.len() as i32)?; // length of 1st packet id + data as VarInt
    socket.write_all(&buffer)?;
    let request: [u8; 2] = [
        1,    // length of 2nd packet id + data as VarInt
        0x00, // 2nd packet id: 0 for request as VarInt
    ];
    socket.write_all(&request)?;
    socket.flush()?;

    let _length = read_varint(&mut socket)?;
    let packet_id = read_varint(&mut socket)?;
    let response_length = read_varint(&mut socket)?;
    if packet_id != 0x00 || response_length < 0 {
        return Err(Error::UnsupportedProtocol);
    }
    let mut response_buffer = vec![0; response_length as usize];
    socket.read_exact(&mut response_buffer)?;

    let raw: RawLatest =
        serde_json::from_slice(&response_buffer).map_err(|_| Error::UnsupportedProtocol)?;
    raw.try_into()
}

fn ping_legacy(hostname: &str, port: u16) -> Result<Response> {
    let mut socket = TcpStream::connect((hostname, port))?;
    let ping_packet = [
        0xfe, // 1st packet id: 0xfe for server list ping
        0x01, // payload: always 1
        0xfa, // 2nd packet id: 0xfa for plugin message
        0x00, 0x0b, // length of following string: always 11 as short,
        0x00, 0x4d, 0x00, 0x43, 0x00, 0x7c, 0x00, 0x50, 0x00, 0x69, 0x00, 0x6e, 0x00, 0x67, 0x00,
        0x48, 0x00, 0x6f, 0x00, 0x73, 0x00, 0x74,
        // MC|PingHost as UTF16-BE
        7,    // length of the rest of the data: 7 + length of hostname
        0x4a, // protocol version: 0x4a for the last version
        0x00, 0x00, // length of hostname: 0 as short
        0x00, 0x00, 0x00, 0x00, // port: 0 as int
    ];
    socket.write_all(&ping_packet)?;
    socket.flush()?;

    let mut buffer = Vec::new();
    socket.read_to_end(&mut buffer)?;
    if buffer.len() <= 3 || buffer[0] != 0xff {
        return Err(Error::UnsupportedProtocol);
    }
    let utf16be: Vec<u16> = buffer[3..]
        .chunks_exact(2)
        .map(|chunk| ((chunk[0] as u16) << 8) | chunk[1] as u16)
        .collect();
    let response = String::from_utf16(&utf16be).map_err(|_| Error::UnsupportedProtocol)?;

    parse_legacy(&response)
}

fn parse_legacy(s: &str) -> Result<Response> {
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
        }),
        _ => Err(Error::UnsupportedProtocol),
    }
}

const LAST_SEVEN_BITS: i32 = 0b0111_1111;
const NEXT_BYTE_EXISTS: u8 = 0b1000_0000;

fn read_varint(stream: &mut impl Read) -> Result<i32> {
    let mut buffer = [0u8];
    let mut result = 0;
    let mut read_count = 0;
    loop {
        stream.read(&mut buffer)?;
        result |= (buffer[0] as i32 & LAST_SEVEN_BITS) << (7 * read_count);

        read_count += 1;
        if read_count > 5 {
            break Err(Error::UnsupportedProtocol);
        } else if (buffer[0] & NEXT_BYTE_EXISTS) == 0 {
            break Ok(result);
        }
    }
}

// bit mask to remove remaining 7 MSB's after right shift
const SEVEN_BITS_SHIFT_MASK: i32 = 0x01_ff_ff_ff;

fn write_varint(sink: &mut impl std::io::Write, mut value: i32) -> Result<()> {
    loop {
        let mut temp = (value & LAST_SEVEN_BITS) as u8;
        // i32 right shift is arithmetic shift (preserves MSB)
        value >>= 7;
        value &= SEVEN_BITS_SHIFT_MASK;
        if value != 0 {
            temp |= NEXT_BYTE_EXISTS;
        }
        sink.write(&[temp])?;
        if value == 0 {
            break Ok(());
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::io::Cursor;
    #[test]
    fn serialize_varint() {
        let mut cursor = Cursor::new(Vec::new());
        let samples = [-2147483648, -1, 0, 1, 2147483647];
        for &i in samples.iter() {
            cursor.set_position(0);
            write_varint(&mut cursor, i).unwrap();
            cursor.set_position(0);
            let deserialized = read_varint(&mut cursor).unwrap();

            assert_eq!(i, deserialized);
        }
    }
}
