//! Get the Gmail POP settings (`users.settings.getPop`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings/getPop>

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

/// I/O-free coroutine getting the Gmail POP access settings
/// (`users.settings.getPop`).
pub struct GmailPopGet {
    send: GmailSend<GmailPopSettings>,
}

impl GmailPopGet {
    /// Builds the `users.settings.getPop` request for the given user.
    pub fn new(auth: &HttpAuthBearer, user_id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail pop settings retrieval");
        trace!("user_id: {user_id:?}");

        let url = gmail_api_url(&["users", user_id, "settings", "pop"])?;
        let send = GmailSend::get(auth, url)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailPopGet {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailPopSettings>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("pop settings retrieved");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
