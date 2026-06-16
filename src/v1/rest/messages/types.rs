use alloc::{string::String, vec::Vec};

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailMessage {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub label_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub internal_date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<GmailMessagePayload>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_estimate: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history_id: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailMessageId {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
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

/// Whether messages carrying a label show up in the message list.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum GmailMessageListVisibility {
    Show,
    Hide,
}

/// Amount of message detail to return (`format` query parameter).
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

/// Source of the internal date when importing or inserting a message
/// (`internalDateSource` query parameter).
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GmailInternalDateSource {
    ReceivedTime,
    DateHeader,
}

impl GmailInternalDateSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReceivedTime => "RECEIVED_TIME",
            Self::DateHeader => "DATE_HEADER",
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
