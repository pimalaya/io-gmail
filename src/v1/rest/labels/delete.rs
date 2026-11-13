//! Delete a Gmail label (`users.labels.delete`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.labels/delete>

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};

use crate::{
    coroutine::*,
    gmail_try,
    v1::send::{GmailNoResponse, GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
};

/// I/O-free coroutine deleting a Gmail label by id (`users.labels.delete`).
pub struct GmailLabelDelete {
    send: GmailSend<GmailNoResponse>,
}

impl GmailLabelDelete {
    /// Builds the `users.labels.delete` request for the given label id.
    pub fn new(auth: &HttpAuthBearer, user_id: &str, id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail label deletion");
        trace!("id: {id:?}");

        let url = gmail_api_url(&["users", user_id, "labels", id])?;
        let send = GmailSend::delete(auth, url)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailLabelDelete {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailNoResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("label deleted");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
