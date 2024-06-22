use byteorder::WriteBytesExt;
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
    fn packet_id(&self) -> u32 {
        0x01
    }

    fn encode<W: Write>(&self, stream: &mut W) -> anyhow::Result<()> {
        stream.write_u64::<LittleEndian>(self.payload)?;

        Ok(())
    }
}

impl PacketDecoder for Ping {
    fn packet_id(&self) -> u32 {
        0x01
    }

    fn decode<R: Read>(_stream: &mut R) -> anyhow::Result<Box<Self>> {
        let start = SystemTime::now();
        let now = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        let payload = now.as_secs();

        Ok(Box::new(Ping { payload }))
    }
}
