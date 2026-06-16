use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailLabel {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, rename = "type")]
    pub label_type: Option<String>,
    #[serde(default)]
    pub message_list_visibility: Option<String>,
    #[serde(default)]
    pub label_list_visibility: Option<String>,
    #[serde(default)]
    pub messages_total: Option<u64>,
    #[serde(default)]
    pub messages_unread: Option<u64>,
    #[serde(default)]
    pub threads_total: Option<u64>,
    #[serde(default)]
    pub threads_unread: Option<u64>,
    #[serde(default)]
    pub color: Option<GmailLabelColor>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailLabelColor {
    #[serde(default)]
    pub text_color: Option<String>,
    #[serde(default)]
    pub background_color: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailLabelsListResponse {
    #[serde(default)]
    pub labels: Vec<GmailLabel>,
}
