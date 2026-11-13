//! Delete a Gmail send-as alias (`users.settings.sendAs.delete`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.sendAs/delete>

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};

use crate::{
    coroutine::*,
    gmail_try,
    v1::send::{GmailNoResponse, GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
};

/// I/O-free coroutine deleting a send-as alias from a Gmail account
/// (`users.settings.sendAs.delete`).
pub struct GmailSendAsDelete {
    send: GmailSend<GmailNoResponse>,
}

impl GmailSendAsDelete {
    /// Builds the `users.settings.sendAs.delete` request for the given alias.
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        send_as_email: &str,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail send-as alias deletion");
        trace!("send_as_email: {send_as_email:?}");

        let url = gmail_api_url(&["users", user_id, "settings", "sendAs", send_as_email])?;
        let send = GmailSend::delete(auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailSendAsDelete {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailNoResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("send-as alias deleted");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
