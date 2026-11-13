//! Get a Gmail user profile (`users.getProfile`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users/getProfile>

use alloc::string::String;

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use serde::{Deserialize, Serialize};

use crate::{
    coroutine::*,
    gmail_try,
    v1::send::{GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
};

/// Aggregated mailbox profile of a Gmail user.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailProfile {
    /// The email address of the user.
    pub email_address: String,
    /// The total number of messages in the mailbox.
    #[serde(default)]
    pub messages_total: Option<u64>,
    /// The total number of threads in the mailbox.
    #[serde(default)]
    pub threads_total: Option<u64>,
    /// The id of the current history record of the mailbox.
    #[serde(default)]
    pub history_id: Option<String>,
}

/// I/O-free coroutine getting the Gmail user profile (`users.getProfile`).
pub struct GmailProfileGet {
    send: GmailSend<GmailProfile>,
}

impl GmailProfileGet {
    /// Builds the `users.getProfile` request for the given user id
    /// (the mailbox owner, usually `me`).
    pub fn new(auth: &HttpAuthBearer, user_id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail profile retrieval");
        trace!("user_id: {user_id:?}");

        let url = gmail_api_url(&["users", user_id, "profile"])?;
        let send = GmailSend::get(auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailProfileGet {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailProfile>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("profile retrieved");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
