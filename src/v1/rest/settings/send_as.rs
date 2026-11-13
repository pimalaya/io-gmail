//! Gmail send-as aliases (`users.settings.sendAs`): list, get, create,
//! update, patch, delete, verify.
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.sendAs>

use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::v1::rest::settings::GmailVerificationStatus;

pub mod create;
pub mod delete;
pub mod get;
pub mod list;
pub mod patch;
pub mod update;
pub mod verify;

/// Send-as alias of a Gmail account.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GmailSendAs {
    /// Email address that appears in the From header of sent mail.
    #[serde(default)]
    pub send_as_email: String,
    /// Display name used in the From header of sent mail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Email address put in the Reply-To header of sent mail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to_address: Option<String>,
    /// HTML signature appended to messages composed with this alias.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    /// Whether this alias is the primary address of the account.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_primary: Option<bool>,
    /// Whether this alias is the default From address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,
    /// Whether Gmail treats this address as an alias of the primary address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub treat_as_alias: Option<bool>,
    /// Optional SMTP service used as outbound relay for this alias.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub smtp_msa: Option<GmailSmtpMsa>,
    /// Verification status of the alias.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_status: Option<GmailVerificationStatus>,
}

/// SMTP relay configuration used to send mail for a send-as alias.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GmailSmtpMsa {
    /// Hostname of the SMTP service.
    #[serde(default)]
    pub host: String,
    /// Port of the SMTP service.
    #[serde(default)]
    pub port: u32,
    /// Username used for authentication against the SMTP service.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// Password used for authentication against the SMTP service.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// Transport security used to connect to the SMTP service.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security_mode: Option<GmailSecurityMode>,
}

/// Transport security mode of an SMTP relay service.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum GmailSecurityMode {
    /// Unspecified security mode.
    SecurityModeUnspecified,
    /// Unsecured communication with the SMTP service.
    None,
    /// Communication secured using SSL from connection start.
    Ssl,
    /// Communication upgraded to a secure channel using STARTTLS.
    Starttls,
    /// A value this crate does not know: read, never written back.
    #[serde(other, skip_serializing)]
    Unknown,
}
