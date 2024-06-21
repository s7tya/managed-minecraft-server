use serde::Deserialize;

use super::raw_json_text::RawJsonText;

#[derive(Deserialize, Debug)]
pub struct Response {
    pub version: Version,
    pub players: Players,
    pub description: RawJsonText,
    pub favicon: String,
    pub modinfo: Option<Modinfo>,
}

#[derive(Deserialize, Debug)]
pub struct Version {
    pub name: String,
    pub protocol: i32,
}

#[derive(Deserialize, Debug)]
pub struct Players {
    pub max: usize,
    pub online: usize,
}

#[derive(Deserialize, Debug)]
pub struct Modinfo {
    #[serde(rename = "modList")]
    pub mod_list: Vec<String>,
    pub r#type: String,
}
