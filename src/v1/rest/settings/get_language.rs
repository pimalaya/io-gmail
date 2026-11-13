//! Get the Gmail language settings (`users.settings.getLanguage`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings/getLanguage>

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

/// I/O-free coroutine getting the Gmail display language settings
/// (`users.settings.getLanguage`).
pub struct GmailLanguageGet {
    send: GmailSend<GmailLanguageSettings>,
}

impl GmailLanguageGet {
    /// Builds the `users.settings.getLanguage` request for the given user.
    pub fn new(auth: &HttpAuthBearer, user_id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail language settings retrieval");
        trace!("user_id: {user_id:?}");

        let url = gmail_api_url(&["users", user_id, "settings", "language"])?;
        let send = GmailSend::get(auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailLanguageGet {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailLanguageSettings>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("language settings retrieved");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
