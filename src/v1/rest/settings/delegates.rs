//! Gmail delegates (`users.settings.delegates`): list, get, create,
//! delete.
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.delegates>

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

/// Delegate granted access to a Gmail account.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GmailDelegate {
    /// Email address of the delegate.
    #[serde(default)]
    pub delegate_email: String,
    /// Verification status of the delegate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_status: Option<GmailVerificationStatus>,
}
