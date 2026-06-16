//! Update the Gmail language settings (`users.settings.updateLanguage`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings/updateLanguage>

use alloc::format;

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
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
        auth: &HttpAuthBearer,
        user_id: &str,
        settings: GmailLanguageSettings,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail language settings update");
        trace!("settings: {settings:?}");

        let url =
            Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/settings/language"))?;
        let send = GmailSend::put_json(auth, url, &settings)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailLanguageUpdate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailLanguageSettings>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail language settings updated");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
