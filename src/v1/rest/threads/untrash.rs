//! Untrash a Gmail thread (`users.threads.untrash`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.threads/untrash>

use alloc::{format, vec::Vec};

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::rest::threads::GmailThread,
    v1::send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
};

/// Gmail REST thread untrashing, wrapping the updated `GmailThread`.
pub struct GmailThreadUntrash {
    send: GmailSend<GmailThread>,
}

impl GmailThreadUntrash {
    pub fn new(auth: &HttpAuthBearer, user_id: &str, id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail thread untrashing");
        trace!("id: {id:?}");

        let url =
            Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/threads/{id}/untrash"))?;
        let send = GmailSend::with_method(auth, "POST", url, None, Vec::new());

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailThreadUntrash {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailThread>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail thread untrashed");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
