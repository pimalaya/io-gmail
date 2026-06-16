//! Gmail history resource types.

use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

use crate::v1::rest::messages::GmailMessage;

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

/// Kind of change to filter the history list on (`historyTypes`).
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum GmailHistoryType {
    MessageAdded,
    MessageDeleted,
    LabelAdded,
    LabelRemoved,
}
