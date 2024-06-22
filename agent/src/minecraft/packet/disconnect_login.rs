use integer_encoding::VarIntWriter;
use serde::Serialize;
use std::io::Write;

use super::PacketEncoder;
use crate::minecraft::raw_json_text::RawJsonText;

#[derive(Debug, Serialize)]
pub struct DisconnectLogin {
    pub reason: RawJsonText,
}

impl PacketEncoder for DisconnectLogin {
    fn packet_id(&self) -> u32 {
        0x00
    }

    fn encode<W: Write>(&self, stream: &mut W) -> anyhow::Result<()> {
        let s = serde_json::to_string(&self.reason)?.into_bytes();
        stream.write_varint(s.len() as u32)?;
        stream.write_all(&s)?;

        Ok(())
    }
}
