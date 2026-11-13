//! Update a Gmail send-as alias (`users.settings.sendAs.update`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.sendAs/update>

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::send_as::GmailSendAs,
        send::{GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
    },
};

/// I/O-free coroutine updating a send-as alias of a Gmail account
/// (`users.settings.sendAs.update`).
pub struct GmailSendAsUpdate {
    send: GmailSend<GmailSendAs>,
}

impl GmailSendAsUpdate {
    /// Builds the `users.settings.sendAs.update` request for the given alias.
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        send_as_email: &str,
        send_as: &GmailSendAs,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail send-as alias update");
        trace!("send_as: {send_as:?}");

        let url = gmail_api_url(&["users", user_id, "settings", "sendAs", send_as_email])?;
        let send = GmailSend::put_json(auth, url, send_as)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailSendAsUpdate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailSendAs>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("send-as alias updated");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
