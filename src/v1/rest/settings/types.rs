use alloc::string::String;

use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailImapSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_expunge: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expunge_behavior: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_folder_size: Option<u32>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailPopSettings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_window: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disposition: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailLanguageSettings {
    #[serde(default)]
    pub display_language: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailAutoForwarding {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email_address: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disposition: Option<String>,
}
