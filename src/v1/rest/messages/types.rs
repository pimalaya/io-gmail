use alloc::{string::String, vec::Vec};

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct GmailMessageId {
    pub id: String,
    #[serde(rename = "threadId", default)]
    pub thread_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailMessage {
    pub id: String,
    #[serde(default)]
    pub thread_id: Option<String>,
    #[serde(default)]
    pub label_ids: Vec<String>,
    #[serde(default)]
    pub internal_date: Option<String>,
    #[serde(default)]
    pub snippet: Option<String>,
    #[serde(default)]
    pub payload: Option<GmailMessagePayload>,
    #[serde(default)]
    pub raw: Option<String>,
    #[serde(default)]
    pub size_estimate: Option<u64>,
    #[serde(default)]
    pub history_id: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailMessagePayload {
    #[serde(default)]
    pub part_id: Option<String>,
    #[serde(default)]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub body: Option<GmailMessagePartBody>,
    #[serde(default)]
    pub filename: String,
    #[serde(default)]
    pub headers: Vec<GmailMessageHeader>,
    #[serde(default)]
    pub parts: Vec<GmailMessagePayload>,
}

impl GmailMessagePayload {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|header| header.name.eq_ignore_ascii_case(name))
            .map(|header| header.value.as_str())
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailMessagePartBody {
    #[serde(default)]
    pub attachment_id: Option<String>,
    #[serde(default)]
    pub size: u32,
    #[serde(default)]
    pub data: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct GmailMessageHeader {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GmailMessageFormat {
    Minimal,
    Full,
    Raw,
    Metadata,
}

impl GmailMessageFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Minimal => "MINIMAL",
            Self::Full => "FULL",
            Self::Raw => "RAW",
            Self::Metadata => "METADATA",
        }
    }
}

pub fn decode_raw(raw: &str) -> Result<Vec<u8>, base64::DecodeError> {
    let normalized: String = raw.chars().filter(|c| !c.is_ascii_whitespace()).collect();
    let normalized = normalized.trim_end_matches('=');
    URL_SAFE_NO_PAD.decode(normalized)
}

pub fn encode_raw(raw: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(raw)
}
