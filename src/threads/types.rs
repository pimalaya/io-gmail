use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailThreadSummary {
    pub id: String,
    #[serde(default)]
    pub snippet: Option<String>,
    #[serde(default)]
    pub history_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailThread {
    pub id: String,
    #[serde(default)]
    pub snippet: Option<String>,
    #[serde(default)]
    pub history_id: Option<String>,
    #[serde(default)]
    pub messages: Vec<crate::messages::GmailMessage>,
}
