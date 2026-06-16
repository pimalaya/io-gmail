use alloc::format;

use log::trace;
use secrecy::SecretString;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::filters::GmailFilter,
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

pub struct GmailFilterGet {
    send: GmailSend<GmailFilter>,
}

impl GmailFilterGet {
    pub fn new(http_auth: &SecretString, user_id: &str, id: &str) -> Result<Self, GmailSendError> {
        trace!("prepare gmail filter {id} retrieval");

        let url =
            Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/settings/filters/{id}"))?;
        let send = GmailSend::get(http_auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailFilterGet {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailFilter>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        trace!("gmail filter retrieved: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
