//! Update the Gmail POP settings (`users.settings.updatePop`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings/updatePop>

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::GmailPopSettings,
        send::{GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
    },
};

/// I/O-free coroutine updating the Gmail POP access settings
/// (`users.settings.updatePop`).
pub struct GmailPopUpdate {
    send: GmailSend<GmailPopSettings>,
}

impl GmailPopUpdate {
    /// Builds the `users.settings.updatePop` request wrapping the given
    /// settings.
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        settings: GmailPopSettings,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail pop settings update");
        trace!("settings: {settings:?}");

        let url = gmail_api_url(&["users", user_id, "settings", "pop"])?;
        let send = GmailSend::put_json(auth, url, &settings)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailPopUpdate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailPopSettings>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("pop settings updated");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
