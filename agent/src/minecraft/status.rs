use serde::{Deserialize, Serialize};

use super::raw_json_text::RawJsonText;

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
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
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Modinfo {
    #[serde(rename = "modList")]
    pub mod_list: Vec<String>,
    pub r#type: String,
}
