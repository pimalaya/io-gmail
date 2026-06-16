//! Update the Gmail POP settings (`users.settings.updatePop`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings/updatePop>

use alloc::format;

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::GmailPopSettings,
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

pub struct GmailPopUpdate {
    send: GmailSend<GmailPopSettings>,
}

impl GmailPopUpdate {
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        settings: GmailPopSettings,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail pop settings update");
        trace!("settings: {settings:?}");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/settings/pop"))?;
        let send = GmailSend::put_json(auth, url, &settings)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailPopUpdate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailPopSettings>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail pop settings updated");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
