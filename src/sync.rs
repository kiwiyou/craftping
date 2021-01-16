//! Provides synchronous, blocking [`ping`](ping) function.
//!
//! The [`ping`](ping) function here sends a ping request, and wait for the server to respond.
//! If you want to send ping in an asynchronous context, see [`tokio`](tokio) module.
use std::{convert::TryInto, net::TcpStream};

use crate::*;

/// Send a ping request to the server and wait for the response.
///
/// See also [`Response`](Response).
///
/// # Examples
///
/// ```no_run
/// use craftping::sync::ping;
///
/// let response = ping("my.server.com", 25565).unwrap();
/// println!(
///     "{} of {} player(s) online",
///     response.online_players,
///     response.max_players,
/// );
/// ```
pub fn ping(hostname: &str, port: u16) -> Result<Response> {
    ping_latest(hostname, port).or_else(|_| ping_legacy(hostname, port))
}

fn ping_latest(hostname: &str, port: u16) -> Result<Response> {
    let mut socket = TcpStream::connect((hostname, port))?;
    let request = build_latest_request(hostname, port)?;
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

    let raw = decode_latest_response(&response_buffer)?;
    raw.try_into()
}

fn ping_legacy(hostname: &str, port: u16) -> Result<Response> {
    let mut socket = TcpStream::connect((hostname, port))?;
    socket.write_all(&LEGACY_REQUEST)?;
    socket.flush()?;

    let mut buffer = Vec::new();
    socket.read_to_end(&mut buffer)?;

    let response = decode_legacy(&buffer)?;
    parse_legacy(&response)
}

fn read_varint(stream: &mut impl Read) -> Result<i32> {
    let mut buffer = [0u8];
    let mut result = 0;
    let mut read_count = 0;
    loop {
        stream.read_exact(&mut buffer)?;
        result |= (buffer[0] as i32 & LAST_SEVEN_BITS) << (7 * read_count);

        read_count += 1;
        if read_count > 5 {
            break Err(Error::UnsupportedProtocol);
        } else if (buffer[0] & NEXT_BYTE_EXISTS) == 0 {
            break Ok(result);
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
