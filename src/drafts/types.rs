use alloc::string::String;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailDraft {
    pub id: String,
    #[serde(default)]
    pub message: Option<crate::messages::GmailMessage>,
}
