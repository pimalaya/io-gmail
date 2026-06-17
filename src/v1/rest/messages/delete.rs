//! Delete a Gmail message (`users.messages.delete`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.messages/delete>

use alloc::format;

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::send::{GMAIL_API_BASE, GmailNoResponse, GmailSend, GmailSendError, GmailSendOutput},
};

/// Gmail REST message permanent deletion, yielding no response body.
pub struct GmailMessageDelete {
    send: GmailSend<GmailNoResponse>,
}

impl GmailMessageDelete {
    pub fn new(auth: &HttpAuthBearer, user_id: &str, id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail message deletion");
        trace!("id: {id:?}");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/messages/{id}"))?;
        let send = GmailSend::delete(auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailMessageDelete {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailNoResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail message deleted");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
