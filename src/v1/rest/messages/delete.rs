//! Delete a Gmail message (`users.messages.delete`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.messages/delete>

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};

use crate::{
    coroutine::*,
    gmail_try,
    v1::send::{GmailNoResponse, GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
};

/// Gmail REST message permanent deletion, yielding no response body.
pub struct GmailMessageDelete {
    send: GmailSend<GmailNoResponse>,
}

impl GmailMessageDelete {
    /// Builds the `users.messages.delete` request for the given
    /// message id.
    pub fn new(auth: &HttpAuthBearer, user_id: &str, id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail message deletion");
        trace!("id: {id:?}");

        let url = gmail_api_url(&["users", user_id, "messages", id])?;
        let send = GmailSend::delete(auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailMessageDelete {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailNoResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("message deleted");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
