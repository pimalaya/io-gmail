use alloc::format;

use log::trace;
use secrecy::SecretString;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::labels::GmailLabelsListResponse,
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

pub struct GmailLabelsList {
    send: GmailSend<GmailLabelsListResponse>,
}

impl GmailLabelsList {
    pub fn new(http_auth: &SecretString, user_id: &str) -> Result<Self, GmailSendError> {
        trace!("prepare gmail labels listing");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/labels"))?;
        let send = GmailSend::get(http_auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailLabelsList {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailLabelsListResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        trace!("gmail labels listed: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
