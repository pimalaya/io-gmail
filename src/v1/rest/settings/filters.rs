//! Gmail filters (`users.settings.filters`): list, get, create, delete.
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.filters>

use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

pub mod create;
pub mod delete;
pub mod get;
pub mod list;

/// Mail filter of a Gmail account.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GmailFilter {
    /// Server-assigned identifier of the filter.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    /// Matching criteria of the filter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub criteria: Option<GmailFilterCriteria>,
    /// Action applied to messages matching the criteria.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<GmailFilterAction>,
}

/// Action applied by a Gmail filter to a matching message.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GmailFilterAction {
    /// Identifiers of labels to add to the message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub add_label_ids: Option<Vec<String>>,
    /// Identifiers of labels to remove from the message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remove_label_ids: Option<Vec<String>>,
    /// Email address the message is forwarded to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forward: Option<String>,
}

/// Conditions matched by a Gmail filter against incoming messages.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GmailFilterCriteria {
    /// Sender display name or email address to match.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    /// Recipient display name or email address to match.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    /// Case-insensitive phrase matched in the message subject.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// Gmail search query the message must match.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    /// Gmail search query the message must not match.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub negated_query: Option<String>,
    /// Whether the message must have an attachment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_attachment: Option<bool>,
    /// Whether chat messages are excluded from matching.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclude_chats: Option<bool>,
    /// Message size in bytes compared with `size_comparison`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u32>,
    /// How the message size relates to the `size` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_comparison: Option<GmailFilterSizeComparison>,
}

/// Comparison applied to the message size in a filter criterion.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum GmailFilterSizeComparison {
    /// No size comparison specified.
    Unspecified,
    /// Matches messages smaller than the given size.
    Smaller,
    /// Matches messages larger than the given size.
    Larger,
    /// A value this crate does not know: read, never written back.
    #[serde(other, skip_serializing)]
    Unknown,
}
