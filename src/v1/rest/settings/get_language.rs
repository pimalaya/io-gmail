//! Get the Gmail language settings (`users.settings.getLanguage`).

use alloc::format;

use log::{debug, trace};
use secrecy::SecretString;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::GmailLanguageSettings,
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

pub struct GmailLanguageGet {
    send: GmailSend<GmailLanguageSettings>,
}

impl GmailLanguageGet {
    pub fn new(http_auth: &SecretString, user_id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail language settings retrieval");
        trace!("user_id: {user_id:?}");

        let url =
            Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/settings/language"))?;
        let send = GmailSend::get(http_auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailLanguageGet {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailLanguageSettings>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail language settings retrieved");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
