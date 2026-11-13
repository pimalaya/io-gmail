//! Get the Gmail vacation settings (`users.settings.getVacation`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings/getVacation>

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::GmailVacationSettings,
        send::{GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
    },
};

/// I/O-free coroutine getting the Gmail vacation responder settings
/// (`users.settings.getVacation`).
pub struct GmailVacationGet {
    send: GmailSend<GmailVacationSettings>,
}

impl GmailVacationGet {
    /// Builds the `users.settings.getVacation` request for the given user.
    pub fn new(auth: &HttpAuthBearer, user_id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail vacation settings retrieval");
        trace!("user_id: {user_id:?}");

        let url = gmail_api_url(&["users", user_id, "settings", "vacation"])?;
        let send = GmailSend::get(auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailVacationGet {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailVacationSettings>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("vacation settings retrieved");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
