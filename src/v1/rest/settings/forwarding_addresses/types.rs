use alloc::string::String;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailForwardingAddress {
    #[serde(default)]
    pub forwarding_email: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_status: Option<String>,
}
