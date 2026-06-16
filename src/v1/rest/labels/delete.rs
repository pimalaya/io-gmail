use alloc::format;

use log::trace;
use secrecy::SecretString;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::send::{GMAIL_API_BASE, GmailNoResponse, GmailSend, GmailSendError, GmailSendOutput},
};

pub struct GmailLabelDelete {
    send: GmailSend<GmailNoResponse>,
}

impl GmailLabelDelete {
    pub fn new(http_auth: &SecretString, user_id: &str, id: &str) -> Result<Self, GmailSendError> {
        trace!("prepare gmail label {id} deletion");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/labels/{id}"))?;
        let send = GmailSend::delete(http_auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailLabelDelete {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailNoResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        trace!("gmail label deleted: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
