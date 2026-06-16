use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

use crate::v1::rest::messages::GmailMessageListVisibility;

#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailLabel {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub label_type: Option<GmailLabelType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_list_visibility: Option<GmailMessageListVisibility>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label_list_visibility: Option<GmailLabelListVisibility>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub messages_total: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub messages_unread: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threads_total: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threads_unread: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<GmailLabelColor>,
}

/// Owner of the label: created by Gmail or by the user.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum GmailLabelType {
    System,
    User,
}

/// Visibility of the label in the label list of the Gmail web client.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum GmailLabelListVisibility {
    LabelShow,
    LabelShowIfUnread,
    LabelHide,
}

/// Text and background colors of a user label, given as hex strings.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailLabelColor {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailLabelsListResponse {
    #[serde(default)]
    pub labels: Vec<GmailLabel>,
}
