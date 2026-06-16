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

pub struct GmailImapUpdate {
    send: GmailSend<GmailImapSettings>,
}

impl GmailImapUpdate {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        settings: GmailImapSettings,
    ) -> Result<Self, GmailSendError> {
        trace!("prepare gmail imap settings update");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/settings/imap"))?;
        let send = GmailSend::put_json(http_auth, url, &settings)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailImapUpdate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailImapSettings>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        trace!("gmail imap settings updated: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
