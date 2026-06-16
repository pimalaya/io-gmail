use alloc::format;

use log::trace;
use secrecy::SecretString;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::GmailVacationSettings,
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

pub struct GmailVacationUpdate {
    send: GmailSend<GmailVacationSettings>,
}

impl GmailVacationUpdate {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        settings: GmailVacationSettings,
    ) -> Result<Self, GmailSendError> {
        trace!("prepare gmail vacation settings update");

        let url =
            Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/settings/vacation"))?;
        let send = GmailSend::put_json(http_auth, url, &settings)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailVacationUpdate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailVacationSettings>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        trace!("gmail vacation settings updated: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
