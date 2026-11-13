//! Get a Gmail label (`users.labels.get`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.labels/get>

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::labels::GmailLabel,
        send::{GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
    },
};

/// I/O-free coroutine getting a Gmail label by id (`users.labels.get`).
pub struct GmailLabelGet {
    send: GmailSend<GmailLabel>,
}

impl GmailLabelGet {
    /// Builds the `users.labels.get` request for the given label id.
    pub fn new(auth: &HttpAuthBearer, user_id: &str, id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail label retrieval");
        trace!("id: {id:?}");

        let url = gmail_api_url(&["users", user_id, "labels", id])?;
        let send = GmailSend::get(auth, url)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailLabelGet {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailLabel>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("label retrieved");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
