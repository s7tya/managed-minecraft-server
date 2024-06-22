use serde::{Deserialize, Serialize};

// https://minecraft.wiki/w/Raw_JSON_text_format
// https://wiki.vg/Text_formatting#Text_components

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum RawJsonText {
    String(String),
    Object(Object),
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Object {
    pub text: String,

    pub r#type: Option<String>,
    pub extra: Option<Vec<RawJsonText>>,

    /* 装飾 */
    pub color: Option<String>,
    // font
    pub bold: Option<bool>,
    // italic
    // underlined
    // strikethrough
    pub obfuscated: Option<bool>,
    // insertion
    // clickEvent
    // hoverEvent
}
