use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::v1::rest::settings::GmailVerificationStatus;

/// Delegate granted access to a Gmail account.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailDelegate {
    #[serde(default)]
    pub delegate_email: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_status: Option<GmailVerificationStatus>,
}
