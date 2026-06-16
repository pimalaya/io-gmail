use alloc::format;

use log::trace;
use secrecy::SecretString;
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
    pub fn new(http_auth: &SecretString, user_id: &str, id: &str) -> Result<Self, GmailSendError> {
        trace!("prepare gmail thread {id} deletion");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/threads/{id}"))?;
        let send = GmailSend::delete(http_auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailThreadDelete {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailNoResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        trace!("gmail thread deleted: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
