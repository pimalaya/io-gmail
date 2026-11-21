//! Gmail mailbox history (`users.history`): the incremental-sync delta
//! since a given `historyId`.
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.history>

use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

use crate::v1::rest::messages::GmailMessage;

#[cfg(feature = "client")]
pub mod list;

/// A Gmail history record resource.
///
/// A record captures a change to the mailbox and may affect multiple
/// messages in multiple ways.
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GmailHistory {
    /// The mailbox sequence id of the history record.
    pub id: String,
    /// The messages changed in this history record.
    #[serde(default)]
    pub messages: Vec<GmailMessage>,
    /// The messages added to the mailbox in this history record.
    #[serde(default)]
    pub messages_added: Vec<GmailHistoryMessage>,
    /// The messages deleted (not trashed) from the mailbox in this
    /// history record.
    #[serde(default)]
    pub messages_deleted: Vec<GmailHistoryMessage>,
    /// The labels added to messages in this history record.
    #[serde(default)]
    pub labels_added: Vec<GmailHistoryLabel>,
    /// The labels removed from messages in this history record.
    #[serde(default)]
    pub labels_removed: Vec<GmailHistoryLabel>,
}

/// A message involved in a history addition or deletion change.
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GmailHistoryMessage {
    /// The affected message.
    pub message: GmailMessage,
}

/// A label change on a message in a history record.
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GmailHistoryLabel {
    /// The affected message.
    pub message: GmailMessage,
    /// The ids of the labels added to or removed from the message.
    #[serde(default)]
    pub label_ids: Vec<String>,
}

/// Kind of change to filter the history list on (`historyTypes`).
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum GmailHistoryType {
    /// A message was added to the mailbox.
    MessageAdded,
    /// A message was deleted (not trashed) from the mailbox.
    MessageDeleted,
    /// A label was added to a message.
    LabelAdded,
    /// A label was removed from a message.
    LabelRemoved,
}
