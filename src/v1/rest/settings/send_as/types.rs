use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::v1::rest::settings::GmailVerificationStatus;

/// Send-as alias of a Gmail account.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailSendAs {
    #[serde(default)]
    pub send_as_email: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to_address: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_primary: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub treat_as_alias: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub smtp_msa: Option<GmailSmtpMsa>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_status: Option<GmailVerificationStatus>,
}

/// SMTP relay configuration used to send mail for a send-as alias.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailSmtpMsa {
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub port: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security_mode: Option<GmailSecurityMode>,
}

/// Transport security mode of an SMTP relay service.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum GmailSecurityMode {
    SecurityModeUnspecified,
    None,
    Ssl,
    Starttls,
}
