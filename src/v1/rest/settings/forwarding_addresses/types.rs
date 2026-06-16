//! Gmail forwarding-address resource types.
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.forwardingAddresses>

use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::v1::rest::settings::GmailVerificationStatus;

/// Forwarding address registered on a Gmail account.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailForwardingAddress {
    #[serde(default)]
    pub forwarding_email: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_status: Option<GmailVerificationStatus>,
}
