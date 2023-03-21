//! Provides synchronous, blocking [`ping`](ping) function.
//!
//! The [`ping`](ping) function here sends a ping request, and wait for the server to respond.
//! If you want to send ping in an asynchronous context, see [`tokio`](tokio) or [`futures`](futures) module.
use std::convert::TryInto;

use crate::*;

/// Send a ping request to the server and wait for the response.
///
/// See also [`Response`](Response).
///
/// # Examples
///
/// ```no_run
/// use craftping::sync::ping;
/// use std::net::TcpStream;
///
/// let hostname = "my.server.com";
/// let port = 25565;
/// let mut stream = TcpStream::connect((hostname, port)).unwrap();
/// let response = ping(&mut stream, hostname, port).unwrap();
/// println!(
///     "{} of {} player(s) online",
///     response.online_players,
///     response.max_players,
/// );
/// ```
pub fn ping<Stream>(stream: &mut Stream, hostname: &str, port: u16) -> Result<Response>
where
    Stream: Read + Write,
{
    ping_latest(stream, hostname, port).or_else(|_| ping_legacy(stream))
}

fn ping_latest<Stream>(stream: &mut Stream, hostname: &str, port: u16) -> Result<Response>
where
    Stream: Read + Write,
{
    let request = build_latest_request(hostname, port)?;
    stream.write_all(&request)?;
    stream.flush()?;

    let _length = read_varint(stream)?;
    let packet_id = read_varint(stream)?;
    let response_length = read_varint(stream)?;
    if packet_id != 0x00 || response_length < 0 {
        return Err(Error::UnsupportedProtocol);
    }
    let mut response_buffer = vec![0; response_length as usize];
    stream.read_exact(&mut response_buffer)?;

    let mut raw = decode_latest_response(&response_buffer)?;
    raw.raw_json = response_buffer;
    raw.try_into()
}

fn ping_legacy<Stream>(stream: &mut Stream) -> Result<Response>
where
    Stream: Read + Write,
{
    stream.write_all(&LEGACY_REQUEST)?;
    stream.flush()?;

    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer)?;

    let response = decode_legacy(&buffer)?;
    parse_legacy(&response, buffer)
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
        let mut buffer = vec![];
        let samples = [-2147483648, -1, 0, 1, 2147483647];
        for &i in samples.iter() {
            buffer.clear();
            write_varint(&mut buffer, i);
            let mut reader = Cursor::new(buffer);
            let deserialized = read_varint(&mut reader).unwrap();

            assert_eq!(i, deserialized);
            buffer = reader.into_inner();
        }
    }
}
