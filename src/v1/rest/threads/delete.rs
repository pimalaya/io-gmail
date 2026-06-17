//! Delete a Gmail thread (`users.threads.delete`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.threads/delete>

use alloc::format;

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::send::{GMAIL_API_BASE, GmailNoResponse, GmailSend, GmailSendError, GmailSendOutput},
};

/// Gmail REST thread permanent deletion, yielding no response body.
pub struct GmailThreadDelete {
    send: GmailSend<GmailNoResponse>,
}

impl GmailThreadDelete {
    pub fn new(auth: &HttpAuthBearer, user_id: &str, id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail thread deletion");
        trace!("id: {id:?}");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/threads/{id}"))?;
        let send = GmailSend::delete(auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailThreadDelete {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailNoResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail thread deleted");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
