use std::io::{Read, Write};

use super::{PacketDecoder, PacketEncoder};

#[derive(Debug)]
pub struct StatusRequest {}

impl PacketEncoder for StatusRequest {
    fn packet_id(&self) -> u32 {
        0x00
    }

    fn encode<W: Write>(&self, _stream: &mut W) -> anyhow::Result<()> {
        Ok(())
    }
}

impl PacketDecoder for StatusRequest {
    fn packet_id(&self) -> u32 {
        0x00
    }

    fn decode<R: Read>(_stream: &mut R) -> anyhow::Result<Box<Self>> {
        Ok(Box::new(StatusRequest {}))
    }
}
