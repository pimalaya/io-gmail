//! Update the Gmail vacation settings (`users.settings.updateVacation`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings/updateVacation>

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

/// I/O-free coroutine updating the Gmail vacation responder settings
/// (`users.settings.updateVacation`).
pub struct GmailVacationUpdate {
    send: GmailSend<GmailVacationSettings>,
}

impl GmailVacationUpdate {
    /// Builds the `users.settings.updateVacation` request wrapping the given
    /// settings.
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        settings: GmailVacationSettings,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail vacation settings update");
        trace!("settings: {settings:?}");

        let url = gmail_api_url(&["users", user_id, "settings", "vacation"])?;
        let send = GmailSend::put_json(auth, url, &settings)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailVacationUpdate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailVacationSettings>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("vacation settings updated");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
