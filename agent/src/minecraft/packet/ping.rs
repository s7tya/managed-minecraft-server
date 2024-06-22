use byteorder::WriteBytesExt;
use integer_encoding::VarIntWriter;
use std::{
    io::{Read, Write},
    time::{SystemTime, UNIX_EPOCH},
};

use byteorder::LittleEndian;

use super::{PacketDecoder, PacketEncoder};

#[derive(Debug)]
pub struct Ping {
    payload: u64,
}

impl PacketEncoder for Ping {
    fn encode<W: Write>(&self, stream: &mut W) -> anyhow::Result<()> {
        stream.write_varint(0x01_u32)?;
        stream.write_u64::<LittleEndian>(self.payload)?;

        Ok(())
    }
}

impl PacketDecoder for Ping {
    fn decode<R: Read>(_stream: &mut R) -> anyhow::Result<Box<Self>> {
        let start = SystemTime::now();
        let now = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        let payload = now.as_secs();

        Ok(Box::new(Ping { payload }))
    }
}
