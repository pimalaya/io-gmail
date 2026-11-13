//! Update the Gmail auto-forwarding settings
//! (`users.settings.updateAutoForwarding`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings/updateAutoForwarding>

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::GmailAutoForwarding,
        send::{GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
    },
};

/// I/O-free coroutine updating the Gmail auto-forwarding settings
/// (`users.settings.updateAutoForwarding`).
pub struct GmailAutoForwardingUpdate {
    send: GmailSend<GmailAutoForwarding>,
}

impl GmailAutoForwardingUpdate {
    /// Builds the `users.settings.updateAutoForwarding` request wrapping the
    /// given settings.
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        settings: GmailAutoForwarding,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail auto-forwarding settings update");
        trace!("settings: {settings:?}");

        let url = gmail_api_url(&["users", user_id, "settings", "autoForwarding"])?;
        let send = GmailSend::put_json(auth, url, &settings)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailAutoForwardingUpdate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailAutoForwarding>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("auto-forwarding settings updated");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
