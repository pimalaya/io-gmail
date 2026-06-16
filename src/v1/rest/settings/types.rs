//! Gmail settings resource types.

use alloc::string::String;

use serde::{Deserialize, Serialize};

/// Vacation auto-reply settings of a Gmail account.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailVacationSettings {
    #[serde(default)]
    pub enable_auto_reply: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_subject: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_body_plain_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_body_html: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restrict_to_contacts: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restrict_to_domain: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
}

/// IMAP access settings of a Gmail account.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailImapSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_expunge: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expunge_behavior: Option<GmailExpungeBehavior>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_folder_size: Option<u32>,
}

/// POP access settings of a Gmail account.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailPopSettings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_window: Option<GmailPopAccessWindow>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disposition: Option<GmailDisposition>,
}

/// Language display settings of a Gmail account.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailLanguageSettings {
    #[serde(default)]
    pub display_language: String,
}

/// Auto-forwarding settings of a Gmail account.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailAutoForwarding {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email_address: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disposition: Option<GmailDisposition>,
}

/// Action applied to a message after it has been forwarded or fetched.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum GmailDisposition {
    DispositionUnspecified,
    LeaveInInbox,
    Archive,
    Trash,
    MarkRead,
}

/// Behavior applied to messages expunged from an IMAP folder.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum GmailExpungeBehavior {
    ExpungeBehaviorUnspecified,
    Archive,
    Trash,
    DeleteForever,
}

/// Range of messages accessible through POP.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum GmailPopAccessWindow {
    AccessWindowUnspecified,
    Disabled,
    FromNowOn,
    AllMail,
}

/// Verification state of an email address owned by a Gmail account.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum GmailVerificationStatus {
    VerificationStatusUnspecified,
    Accepted,
    Pending,
    Rejected,
    Expired,
}
