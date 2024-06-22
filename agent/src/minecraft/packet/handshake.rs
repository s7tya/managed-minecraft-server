use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use integer_encoding::{VarIntReader, VarIntWriter};
use std::io::{Cursor, Read, Write};

use super::{PacketDecoder, PacketEncoder};

#[derive(Debug)]
pub struct Handshake {
    pub version: i32,
    pub host: String,
    pub port: u16,
    pub next_status: i32,
}

impl PacketEncoder for Handshake {
    fn packet_id(&self) -> u32 {
        0x00
    }

    fn encode<W: Write>(&self, stream: &mut W) -> anyhow::Result<()> {
        stream.write_varint(self.version as u32)?;

        let host_bytes = self.host.as_bytes();
        stream.write_varint(host_bytes.len() as u32)?;
        stream.write_all(host_bytes)?;

        stream.write_u16::<BigEndian>(self.port)?;

        stream.write_varint(self.next_status as u32)?;

        Ok(())
    }
}

impl PacketDecoder for Handshake {
    fn packet_id(&self) -> u32 {
        0x00
    }

    fn decode<R: Read>(stream: &mut R) -> anyhow::Result<Box<Self>> {
        let version: u32 = stream.read_varint()?;

        let host_len: u32 = stream.read_varint()?;
        let mut host_buf = vec![0_u8; host_len as usize];
        stream.read_exact(&mut host_buf)?;

        let mut host_buf_cur = Cursor::new(host_buf);
        let mut host = String::new();
        host_buf_cur.read_to_string(&mut host)?;

        let port = stream.read_u16::<BigEndian>()?;

        let next_status: u32 = stream.read_varint()?;

        Ok(Box::new(Handshake {
            version: version as i32,
            host,
            port,
            next_status: next_status as i32,
        }))
    }
}
