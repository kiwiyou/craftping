//! Provides asynchronous [`ping`](ping) function.
//!
//! The [`ping`](ping) function here sends a ping request, and returns a [`Future`](std::future::Future) resolves to a result of [`Response`](Response).
//! If you want to send ping synchronously, see [`sync`](sync) module.
use std::convert::TryInto;

use ::tokio::io::{AsyncReadExt, AsyncWriteExt};
use ::tokio::net::TcpStream;

use crate::*;

/// Send a ping request to the server and return a future response.
///
/// See also [`Response`](Response).
///
/// # Examples
///
/// ```no_run
/// use craftping::tokio::ping;
///
/// # async fn run() {
/// let response = ping("my.server.com", 25565).await.unwrap();
/// println!(
///     "{} of {} player(s) online",
///     response.online_players,
///     response.max_players,
/// );
/// # }
/// ```
pub async fn ping(hostname: &str, port: u16) -> Result<Response> {
    match ping_latest(hostname, port).await {
        ok @ Ok(_) => ok,
        Err(_) => ping_legacy(hostname, port).await,
    }
}

async fn ping_latest(hostname: &str, port: u16) -> Result<Response> {
    let mut socket = TcpStream::connect((hostname, port)).await?;
    let request = build_latest_request(hostname, port)?;
    socket.write_all(&request).await?;
    socket.flush().await?;

    let _length = read_varint(&mut socket).await?;
    let packet_id = read_varint(&mut socket).await?;
    let response_length = read_varint(&mut socket).await?;
    if packet_id != 0x00 || response_length < 0 {
        return Err(Error::UnsupportedProtocol);
    }
    let mut response_buffer = vec![0; response_length as usize];
    socket.read_exact(&mut response_buffer).await?;

    let raw = decode_latest_response(&response_buffer)?;
    raw.try_into()
}

async fn ping_legacy(hostname: &str, port: u16) -> Result<Response> {
    let mut socket = TcpStream::connect((hostname, port)).await?;
    socket.write_all(&LEGACY_REQUEST).await?;
    socket.flush().await?;

    let mut buffer = Vec::new();
    socket.read_to_end(&mut buffer).await?;

    let response = decode_legacy(&buffer)?;
    parse_legacy(&response)
}

async fn read_varint(stream: &mut (impl AsyncReadExt + Unpin)) -> Result<i32> {
    let mut buffer = [0u8];
    let mut result = 0;
    let mut read_count = 0i32;
    loop {
        stream.read_exact(&mut buffer).await?;
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
        let runtime = ::tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let mut cursor = Cursor::new(Vec::new());
        let samples = [-2147483648, -1, 0, 1, 2147483647];
        for &i in samples.iter() {
            cursor.set_position(0);
            write_varint(&mut cursor, i).unwrap();
            cursor.set_position(0);
            let task = read_varint(&mut cursor);

            let deserialized = runtime.block_on(task).unwrap();

            assert_eq!(i, deserialized);
        }
    }
}
