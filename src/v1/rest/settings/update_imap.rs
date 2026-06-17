//! Update the Gmail IMAP settings (`users.settings.updateImap`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings/updateImap>

use alloc::format;

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::GmailImapSettings,
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

pub struct GmailImapUpdate {
    send: GmailSend<GmailImapSettings>,
}

impl GmailImapUpdate {
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        settings: GmailImapSettings,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail imap settings update");
        trace!("settings: {settings:?}");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/settings/imap"))?;
        let send = GmailSend::put_json(auth, url, &settings)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailImapUpdate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailImapSettings>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail imap settings updated");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
