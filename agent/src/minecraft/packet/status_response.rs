use integer_encoding::VarIntWriter;
use serde::{Deserialize, Serialize};

use crate::minecraft::raw_json_text::RawJsonText;

use super::PacketEncoder;

#[derive(Serialize, Deserialize, Debug)]
pub struct StatusResponse {
    pub version: Version,
    pub players: Players,
    pub description: RawJsonText,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modinfo: Option<Modinfo>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Version {
    pub name: String,
    pub protocol: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Players {
    pub max: usize,
    pub online: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample: Option<Vec<SamplePlayer>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SamplePlayer {
    pub name: String,
    pub id: uuid::Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Modinfo {
    #[serde(rename = "modList")]
    pub mod_list: Vec<String>,
    pub r#type: String,
}

impl PacketEncoder for StatusResponse {
    fn packet_id(&self) -> u32 {
        0x00
    }

    fn encode<W: std::io::Write>(&self, stream: &mut W) -> anyhow::Result<()> {
        let s = serde_json::to_string(&self)?.into_bytes();
        stream.write_varint(s.len() as u32)?;
        stream.write_all(&s)?;

        Ok(())
    }
}
