//! Set up Gmail push notifications (`users.watch`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users/watch>

use alloc::{string::String, vec::Vec};

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use serde::{Deserialize, Serialize};

use crate::{
    coroutine::*,
    gmail_try,
    v1::send::{GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
};

/// Push-notification watch request body (`users.watch`).
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailWatchRequest {
    /// The fully qualified Cloud Pub/Sub topic to publish
    /// notifications to.
    pub topic_name: String,
    /// The label ids to restrict notifications about.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub label_ids: Vec<String>,
    /// The filtering behavior applied to the label ids.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label_filter_behavior: Option<GmailLabelFilterBehavior>,
}

/// Whether a watch includes or excludes its label IDs.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum GmailLabelFilterBehavior {
    /// Only changes on the listed labels trigger a notification.
    Include,
    /// Changes on the listed labels never trigger a notification.
    Exclude,
}

/// Result of establishing a watch (`users.watch`).
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailWatchResponse {
    /// The id of the current history record of the mailbox.
    #[serde(default)]
    pub history_id: Option<String>,
    /// The expiration time of the watch as epoch milliseconds.
    #[serde(default)]
    pub expiration: Option<String>,
}

/// I/O-free coroutine setting up Gmail push notifications (`users.watch`).
pub struct GmailWatch {
    send: GmailSend<GmailWatchResponse>,
}

impl GmailWatch {
    /// Builds the `users.watch` request from the given [`GmailWatchRequest`].
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        request: &GmailWatchRequest,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail watch");
        trace!("request: {request:?}");

        let url = gmail_api_url(&["users", user_id, "watch"])?;
        let send = GmailSend::post_json(auth, url, request)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailWatch {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailWatchResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("watch established");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
