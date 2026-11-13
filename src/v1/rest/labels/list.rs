//! List the Gmail labels (`users.labels.list`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.labels/list>

use alloc::vec::Vec;

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use serde::{Deserialize, Serialize};

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::labels::GmailLabel,
        send::{GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
    },
};

/// Response body of `users.labels.list`.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailLabelsListResponse {
    /// The list of labels.
    #[serde(default)]
    pub labels: Vec<GmailLabel>,
}

/// I/O-free coroutine listing the Gmail labels (`users.labels.list`).
pub struct GmailLabelsList {
    send: GmailSend<GmailLabelsListResponse>,
}

impl GmailLabelsList {
    /// Builds the `users.labels.list` request for the given user id
    /// (the mailbox owner, usually `me`).
    pub fn new(auth: &HttpAuthBearer, user_id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail labels listing");
        trace!("user_id: {user_id:?}");

        let url = gmail_api_url(&["users", user_id, "labels"])?;
        let send = GmailSend::get(auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailLabelsList {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailLabelsListResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("labels listed");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
