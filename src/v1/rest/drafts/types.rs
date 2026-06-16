//! Gmail draft resource types.
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.drafts>

use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::v1::rest::messages::GmailMessage;

/// Gmail REST draft resource (an id plus its draft message).
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailDraft {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<GmailMessage>,
}
