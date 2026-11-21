//! Gmail forwarding addresses (`users.settings.forwardingAddresses`):
//! list, get, create, delete.
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.forwardingAddresses>

use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::v1::rest::settings::GmailVerificationStatus;

#[cfg(feature = "client")]
pub mod create;
#[cfg(feature = "client")]
pub mod delete;
#[cfg(feature = "client")]
pub mod get;
#[cfg(feature = "client")]
pub mod list;

/// Forwarding address registered on a Gmail account.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GmailForwardingAddress {
    /// Email address messages can be forwarded to.
    #[serde(default)]
    pub forwarding_email: String,
    /// Verification status of the forwarding address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_status: Option<GmailVerificationStatus>,
}
