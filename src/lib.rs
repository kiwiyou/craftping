#![crate_name = "craftping"]

use std::io::{Cursor, Error, ErrorKind, Read, Result, Write};

trait ReadPacket {
    fn read_packet(&mut self) -> Result<CraftPacket>;
    fn read_varint(&mut self) -> Result<(i32, usize)>;
}

struct CraftPacket(i32, Box<[u8]>);

impl<R: Read> ReadPacket for R {
    fn read_packet(&mut self) -> Result<CraftPacket> {
        let length = self.read_varint()?.0 as usize;
        let (packet_id, id_length) = self.read_varint()?;
        let mut buffer = vec![0; length - id_length];
        self.read_exact(&mut buffer)?;
        Ok(CraftPacket(packet_id, buffer.into_boxed_slice()))
    }

    fn read_varint(&mut self) -> Result<(i32, usize)> {
        let mut read = 0;
        let mut result = 0;
        let mut buffer = [0];
        loop {
            self.read_exact(&mut buffer)?;
            let value = buffer[0] & 0b0111_1111;
            result |= (value as i32) << (7 * read);
            read += 1;
            if read > 5 {
                return Err(Error::from(ErrorKind::InvalidData));
            }
            if (buffer[0] & 0b1000_0000) == 0 {
                return Ok((result, read as usize));
            }
        }
    }
}

trait WritePacket {
    fn write_packet(&mut self, packet: CraftPacket) -> Result<()>;
    fn write_varint(&mut self, int: i32) -> Result<usize>;
}

impl<W: Write> WritePacket for W {
    fn write_packet(&mut self, packet: CraftPacket) -> Result<()> {
        let mut buffer: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        buffer.write_varint(packet.0)?;
        buffer.write_all(&packet.1)?;
        buffer.set_position(0);
        let mut inner = buffer.into_inner();
        self.write_varint(inner.len() as i32)?;
        self.write_all(&mut inner)?;
        Ok(())
    }

    fn write_varint(&mut self, int: i32) -> Result<usize> {
        let mut int = (int as i64) & 0xFFFF_FFFF;
        let mut written = 0;
        let mut buffer = [0; 5];
        loop {
            let temp = (int & 0b0111_1111) as u8;
            int >>= 7;
            if int != 0 {
                buffer[written] = temp | 0b1000_0000;
            } else {
                buffer[written] = temp;
            }
            written += 1;
            if int == 0 {
                break;
            }
        }
        self.write_all(&mut buffer[0..written])?;
        Ok(written)
    }
}

/// Execute a server list ping (for version > 1.6) Synchronously.
/// Returns ping response in JSON format.
///
/// # Arguments
/// * `addr` - A string slice that represents the host without port
/// * `port` - port number to ping on
pub fn ping(addr: &str, port: u16) -> Result<String> {
    use std::net::TcpStream;
    let mut stream = TcpStream::connect(format!("{}:{}", addr, port))?;
    {
        let mut buffer = Cursor::new(Vec::<u8>::new());
        buffer.write_varint(-1)?;
        buffer.write_varint(addr.len() as i32)?;
        buffer.write_all(addr.as_bytes())?;
        let u16_buffer = [(port >> 8) as u8, (port & 0xFF) as u8];
        buffer.write_all(&u16_buffer)?;
        buffer.write_varint(1)?;
        let consumed = buffer.into_inner();
        stream.write_packet(CraftPacket(0, consumed.into_boxed_slice()))?;
    }
    stream.write_packet(CraftPacket(0, Box::new([])))?;
    stream.flush()?;
    {
        let CraftPacket(packet_id, data) = stream.read_packet()?;
        if packet_id != 0 {
            return Err(Error::from(ErrorKind::InvalidData));
        }
        let mut reader = Cursor::new(data);
        let (length, _) = reader.read_varint()?;
        let mut buffer = vec![0; length as usize];
        reader.read_exact(&mut buffer)?;
        String::from_utf8(buffer).map_err(|e| Error::new(ErrorKind::Other, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_varint() {
        let test_cases: [(i32, Vec<u8>); 5] = [
            (0, vec![0x00]),
            (1, vec![0x01]),
            (127, vec![0x7f]),
            (128, vec![0x80, 0x01]),
            (-2147483648, vec![0x80, 0x80, 0x80, 0x80, 0x08]),
        ];

        for (num, result) in &test_cases {
            let mut buffer = Cursor::new(Vec::new());
            buffer.write_varint(*num).unwrap();
            let inner = buffer.into_inner();
            assert_eq!(&inner, result);
            let mut buffer = Cursor::new(inner);
            let (varint, _) = buffer.read_varint().unwrap();
            assert_eq!(varint, *num);
        }
    }
}
