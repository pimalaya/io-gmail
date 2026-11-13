//! Get a Gmail send-as alias (`users.settings.sendAs.get`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.sendAs/get>

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

/// I/O-free coroutine getting a send-as alias of a Gmail account
/// (`users.settings.sendAs.get`).
pub struct GmailSendAsGet {
    send: GmailSend<GmailSendAs>,
}

impl GmailSendAsGet {
    /// Builds the `users.settings.sendAs.get` request for the given alias.
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        send_as_email: &str,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail send-as alias retrieval");
        trace!("send_as_email: {send_as_email:?}");

        let url = gmail_api_url(&["users", user_id, "settings", "sendAs", send_as_email])?;
        let send = GmailSend::get(auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailSendAsGet {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailSendAs>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("send-as alias retrieved");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
