use integer_encoding::{VarIntReader, VarIntWriter};
use std::io::{Read, Write};

use super::{PacketDecoder, PacketEncoder};

#[derive(Debug)]
pub struct StatusRequest {}

impl PacketEncoder for StatusRequest {
    fn encode<W: Write>(&self, stream: &mut W) -> anyhow::Result<()> {
        stream.write_varint(0x00_u32)?;

        Ok(())
    }
}

impl PacketDecoder for StatusRequest {
    fn decode<R: Read>(stream: &mut R) -> anyhow::Result<Box<Self>> {
        let packet_id: u32 = stream.read_varint()?;
        if packet_id != 0x00 {
            return Err(anyhow::anyhow!(
                "Unsupported protocol: packet_id={:02X?}",
                packet_id,
            ));
        }

        Ok(Box::new(StatusRequest {}))
    }
}
