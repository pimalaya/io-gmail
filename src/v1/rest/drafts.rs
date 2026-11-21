//! Gmail drafts (`users.drafts`): list, get, create, update, send, delete.
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.drafts>

use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::v1::rest::messages::GmailMessage;

#[cfg(feature = "client")]
pub mod create;
#[cfg(feature = "client")]
pub mod delete;
#[cfg(feature = "client")]
pub mod get;
#[cfg(feature = "client")]
pub mod list;
#[cfg(feature = "client")]
pub mod send;
#[cfg(feature = "client")]
pub mod update;

/// Gmail REST draft resource (an id plus its draft message).
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GmailDraft {
    /// Immutable identifier of the draft.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    /// Message content of the draft.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<GmailMessage>,
}
