//! Delete a Gmail draft (`users.drafts.delete`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.drafts/delete>

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};

use crate::{
    coroutine::*,
    gmail_try,
    v1::send::{GmailNoResponse, GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
};

/// Gmail REST draft deletion, yielding no response body.
pub struct GmailDraftDelete {
    send: GmailSend<GmailNoResponse>,
}

impl GmailDraftDelete {
    /// Builds the `users.drafts.delete` request for the given draft id.
    pub fn new(auth: &HttpAuthBearer, user_id: &str, id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail draft deletion");
        trace!("id: {id:?}");

        let url = gmail_api_url(&["users", user_id, "drafts", id])?;
        let send = GmailSend::delete(auth, url)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailDraftDelete {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailNoResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("draft deleted");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
