//! Update the Gmail vacation settings (`users.settings.updateVacation`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings/updateVacation>

use alloc::format;

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::GmailVacationSettings,
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

pub struct GmailVacationUpdate {
    send: GmailSend<GmailVacationSettings>,
}

impl GmailVacationUpdate {
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        settings: GmailVacationSettings,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail vacation settings update");
        trace!("settings: {settings:?}");

        let url =
            Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/settings/vacation"))?;
        let send = GmailSend::put_json(auth, url, &settings)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailVacationUpdate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailVacationSettings>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail vacation settings updated");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
