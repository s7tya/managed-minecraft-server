use serde::Deserialize;

// https://minecraft.wiki/w/Raw_JSON_text_format
// https://wiki.vg/Text_formatting#Text_components

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum RawJsonText {
    String(String),
    Object(Object),
}

#[derive(Deserialize, Debug)]
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
