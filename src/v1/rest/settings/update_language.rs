//! Update the Gmail language settings (`users.settings.updateLanguage`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings/updateLanguage>

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::GmailLanguageSettings,
        send::{GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
    },
};

/// I/O-free coroutine updating the Gmail display language settings
/// (`users.settings.updateLanguage`).
pub struct GmailLanguageUpdate {
    send: GmailSend<GmailLanguageSettings>,
}

impl GmailLanguageUpdate {
    /// Builds the `users.settings.updateLanguage` request wrapping the given
    /// settings.
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        settings: GmailLanguageSettings,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail language settings update");
        trace!("settings: {settings:?}");

        let url = gmail_api_url(&["users", user_id, "settings", "language"])?;
        let send = GmailSend::put_json(auth, url, &settings)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailLanguageUpdate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailLanguageSettings>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("language settings updated");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
