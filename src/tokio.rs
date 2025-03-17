//! Provides asynchronous [`ping`] function. (especially for tokio streams)
//!
//! The [`ping`] function here sends a ping request, and returns a [`Future`] resolves to a result of [`Response`].
//! If you want to send ping synchronously, see [`sync`] module.
use std::convert::TryInto;

use ::tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::*;

/// Send a ping request to the server and return a future response.
///
/// See also [`Response`].
///
/// # Examples
///
/// ```no_run
/// use craftping::tokio::ping;
/// use tokio::net::TcpStream;
///
/// # async fn run() {
/// let hostname = "my.server.com";
/// let port = 25565;
/// let mut stream = TcpStream::connect((hostname, port)).await.unwrap();
/// let response = ping(&mut stream, hostname, port).await.unwrap();
/// println!(
///     "{} of {} player(s) online",
///     response.online_players,
///     response.max_players,
/// );
/// # }
/// ```
pub async fn ping<Stream>(stream: &mut Stream, hostname: &str, port: u16) -> Result<Response>
where
    Stream: AsyncRead + AsyncWrite + Unpin,
{
    match ping_latest(stream, hostname, port).await {
        ok @ Ok(_) => ok,
        Err(_) => ping_legacy(stream).await,
    }
}

async fn ping_latest<Stream>(stream: &mut Stream, hostname: &str, port: u16) -> Result<Response>
where
    Stream: AsyncRead + AsyncWrite + Unpin,
{
    let request = build_latest_request(hostname, port)?;
    stream.write_all(&request).await?;
    stream.flush().await?;

    let _length = read_varint(stream).await?;
    let packet_id = read_varint(stream).await?;
    let response_length = read_varint(stream).await?;
    if packet_id != 0x00 || response_length < 0 {
        return Err(Error::UnsupportedProtocol);
    }
    let mut response_buffer = vec![0; response_length as usize];
    stream.read_exact(&mut response_buffer).await?;

    let mut raw = decode_latest_response(&response_buffer)?;
    raw.raw_json = response_buffer;
    raw.try_into()
}

async fn ping_legacy<Stream>(stream: &mut Stream) -> Result<Response>
where
    Stream: AsyncRead + AsyncWrite + Unpin,
{
    stream.write_all(&LEGACY_REQUEST).await?;
    stream.flush().await?;

    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).await?;

    let response = decode_legacy(&buffer)?;
    parse_legacy(&response, buffer)
}

async fn read_varint<Stream>(stream: &mut Stream) -> Result<i32>
where
    Stream: AsyncRead + Unpin,
{
    let mut buffer = [0u8];
    let mut result = 0;
    let mut read_count = 0u32;
    loop {
        stream.read_exact(&mut buffer).await?;
        result |= (buffer[0] as i32 & LAST_SEVEN_BITS)
            .checked_shl(7 * read_count)
            .ok_or(Error::UnsupportedProtocol)?;

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
        let mut buffer = vec![];
        let samples = [-2147483648, -1, 0, 1, 2147483647];
        for &i in samples.iter() {
            buffer.clear();
            write_varint(&mut buffer, i);
            let mut reader = Cursor::new(buffer);
            let task = read_varint(&mut reader);

            let deserialized = runtime.block_on(task).unwrap();

            assert_eq!(i, deserialized);
            buffer = reader.into_inner();
        }
    }
}
