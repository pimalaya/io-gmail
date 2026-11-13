//! Get the Gmail IMAP settings (`users.settings.getImap`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings/getImap>

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

/// I/O-free coroutine getting the Gmail IMAP access settings
/// (`users.settings.getImap`).
pub struct GmailImapGet {
    send: GmailSend<GmailImapSettings>,
}

impl GmailImapGet {
    /// Builds the `users.settings.getImap` request for the given user.
    pub fn new(auth: &HttpAuthBearer, user_id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail imap settings retrieval");
        trace!("user_id: {user_id:?}");

        let url = gmail_api_url(&["users", user_id, "settings", "imap"])?;
        let send = GmailSend::get(auth, url)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailImapGet {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailImapSettings>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("imap settings retrieved");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
