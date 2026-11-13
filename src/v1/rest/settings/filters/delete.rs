//! Delete a Gmail filter (`users.settings.filters.delete`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.filters/delete>

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};

use crate::{
    coroutine::*,
    gmail_try,
    v1::send::{GmailNoResponse, GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
};

/// I/O-free coroutine deleting a filter from a Gmail account
/// (`users.settings.filters.delete`).
pub struct GmailFilterDelete {
    send: GmailSend<GmailNoResponse>,
}

impl GmailFilterDelete {
    /// Builds the `users.settings.filters.delete` request for the given
    /// filter id.
    pub fn new(auth: &HttpAuthBearer, user_id: &str, id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail filter deletion");
        trace!("id: {id:?}");

        let url = gmail_api_url(&["users", user_id, "settings", "filters", id])?;
        let send = GmailSend::delete(auth, url)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailFilterDelete {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailNoResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("filter deleted");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
