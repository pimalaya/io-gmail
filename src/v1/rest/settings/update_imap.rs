//! Update the Gmail IMAP settings (`users.settings.updateImap`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings/updateImap>

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::GmailImapSettings,
        send::{GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
    },
};

/// I/O-free coroutine updating the Gmail IMAP access settings
/// (`users.settings.updateImap`).
pub struct GmailImapUpdate {
    send: GmailSend<GmailImapSettings>,
}

impl GmailImapUpdate {
    /// Builds the `users.settings.updateImap` request wrapping the given
    /// settings.
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        settings: GmailImapSettings,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail imap settings update");
        trace!("settings: {settings:?}");

        let url = gmail_api_url(&["users", user_id, "settings", "imap"])?;
        let send = GmailSend::put_json(auth, url, &settings)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailImapUpdate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailImapSettings>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("imap settings updated");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
