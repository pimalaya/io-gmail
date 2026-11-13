//! Get a Gmail delegate (`users.settings.delegates.get`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.delegates/get>

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::delegates::GmailDelegate,
        send::{GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
    },
};

/// I/O-free coroutine getting a delegate of a Gmail account
/// (`users.settings.delegates.get`).
pub struct GmailDelegateGet {
    send: GmailSend<GmailDelegate>,
}

impl GmailDelegateGet {
    /// Builds the `users.settings.delegates.get` request for the given
    /// delegate email.
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        delegate_email: &str,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail delegate retrieval");
        trace!("delegate_email: {delegate_email:?}");

        let url = gmail_api_url(&["users", user_id, "settings", "delegates", delegate_email])?;
        let send = GmailSend::get(auth, url)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailDelegateGet {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailDelegate>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("delegate retrieved");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
