use alloc::format;

use log::trace;
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

pub struct GmailLanguageUpdate {
    send: GmailSend<GmailLanguageSettings>,
}

impl GmailLanguageUpdate {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        settings: GmailLanguageSettings,
    ) -> Result<Self, GmailSendError> {
        trace!("prepare gmail language settings update");

        let url =
            Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/settings/language"))?;
        let send = GmailSend::put_json(http_auth, url, &settings)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailLanguageUpdate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailLanguageSettings>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        trace!("gmail language settings updated: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
