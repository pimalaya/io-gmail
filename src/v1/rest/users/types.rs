//! Gmail user-level resource types (profile, watch).

use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

/// Aggregated mailbox profile of a Gmail user.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailProfile {
    pub email_address: String,
    #[serde(default)]
    pub messages_total: Option<u64>,
    #[serde(default)]
    pub threads_total: Option<u64>,
    #[serde(default)]
    pub history_id: Option<String>,
}

/// Push-notification watch request body (`users.watch`).
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailWatchRequest {
    pub topic_name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub label_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label_filter_behavior: Option<GmailLabelFilterBehavior>,
}

/// Result of establishing a watch (`users.watch`).
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailWatchResponse {
    #[serde(default)]
    pub history_id: Option<String>,
    #[serde(default)]
    pub expiration: Option<String>,
}

/// Whether a watch includes or excludes its label IDs.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum GmailLabelFilterBehavior {
    Include,
    Exclude,
}
