use alloc::format;

use log::trace;
use secrecy::SecretString;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::GmailImapSettings,
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

pub struct GmailImapGet {
    send: GmailSend<GmailImapSettings>,
}

impl GmailImapGet {
    pub fn new(http_auth: &SecretString, user_id: &str) -> Result<Self, GmailSendError> {
        trace!("prepare gmail imap settings retrieval");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/settings/imap"))?;
        let send = GmailSend::get(http_auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailImapGet {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailImapSettings>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        trace!("gmail imap settings retrieved: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
