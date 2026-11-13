//! Delete a Gmail delegate (`users.settings.delegates.delete`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.delegates/delete>

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};

use crate::{
    coroutine::*,
    gmail_try,
    v1::send::{GmailNoResponse, GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
};

/// I/O-free coroutine deleting a delegate from a Gmail account
/// (`users.settings.delegates.delete`).
pub struct GmailDelegateDelete {
    send: GmailSend<GmailNoResponse>,
}

impl GmailDelegateDelete {
    /// Builds the `users.settings.delegates.delete` request for the given
    /// delegate email.
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        delegate_email: &str,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail delegate deletion");
        trace!("delegate_email: {delegate_email:?}");

        let url = gmail_api_url(&["users", user_id, "settings", "delegates", delegate_email])?;
        let send = GmailSend::delete(auth, url)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailDelegateDelete {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailNoResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("delegate deleted");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
