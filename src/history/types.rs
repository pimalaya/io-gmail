use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

use crate::messages::GmailMessage;

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailHistory {
    pub id: String,
    #[serde(default)]
    pub messages: Vec<GmailMessage>,
    #[serde(default)]
    pub messages_added: Vec<GmailHistoryMessage>,
    #[serde(default)]
    pub messages_deleted: Vec<GmailHistoryMessage>,
    #[serde(default)]
    pub labels_added: Vec<GmailHistoryLabel>,
    #[serde(default)]
    pub labels_removed: Vec<GmailHistoryLabel>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailHistoryMessage {
    pub message: GmailMessage,
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailHistoryLabel {
    pub message: GmailMessage,
    #[serde(default)]
    pub label_ids: Vec<String>,
}
