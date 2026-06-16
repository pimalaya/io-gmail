use alloc::format;

use log::trace;
use secrecy::SecretString;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::labels::GmailLabel,
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

pub struct GmailLabelGet {
    send: GmailSend<GmailLabel>,
}

impl GmailLabelGet {
    pub fn new(http_auth: &SecretString, user_id: &str, id: &str) -> Result<Self, GmailSendError> {
        trace!("prepare gmail label {id} retrieval");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/labels/{id}"))?;
        let send = GmailSend::get(http_auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailLabelGet {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailLabel>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        trace!("gmail label retrieved: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
