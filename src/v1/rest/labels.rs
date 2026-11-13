//! Gmail labels (`users.labels`): list, get, create, patch, update,
//! delete.
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.labels>

use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::v1::rest::messages::GmailMessageListVisibility;

pub mod create;
pub mod delete;
pub mod get;
pub mod list;
pub mod patch;
pub mod update;

/// A Gmail label resource.
///
/// Labels are used to categorize messages and threads within the
/// user's mailbox.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GmailLabel {
    /// The immutable id of the label.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    /// The display name of the label.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// The owner type of the label, serialized as `type`.
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub label_type: Option<GmailLabelType>,
    /// The visibility of messages with this label in the message list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_list_visibility: Option<GmailMessageListVisibility>,
    /// The visibility of the label in the label list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label_list_visibility: Option<GmailLabelListVisibility>,
    /// The total number of messages with the label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub messages_total: Option<u64>,
    /// The number of unread messages with the label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub messages_unread: Option<u64>,
    /// The total number of threads with the label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threads_total: Option<u64>,
    /// The number of unread threads with the label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threads_unread: Option<u64>,
    /// The color of the label; only available for user labels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<GmailLabelColor>,
}

/// Text and background colors of a user label, given as hex strings.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GmailLabelColor {
    /// The text color of the label as a hex string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_color: Option<String>,
    /// The background color of the label as a hex string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
}

/// Visibility of the label in the label list of the Gmail web client.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum GmailLabelListVisibility {
    /// The label always shows in the label list.
    LabelShow,
    /// The label shows in the label list only when unread.
    LabelShowIfUnread,
    /// The label never shows in the label list.
    LabelHide,
    /// A value this crate does not know: read, never written back.
    #[serde(other, skip_serializing)]
    Unknown,
}

/// Owner of the label: created by Gmail or by the user.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum GmailLabelType {
    /// A label created by Gmail, like INBOX or TRASH.
    System,
    /// A label created by the user.
    User,
    /// A value this crate does not know: read, never written back.
    #[serde(other, skip_serializing)]
    Unknown,
}
