//! Get the Gmail IMAP settings (`users.settings.getImap`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings/getImap>

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

pub struct GmailImapGet {
    send: GmailSend<GmailImapSettings>,
}

impl GmailImapGet {
    pub fn new(auth: &HttpAuthBearer, user_id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail imap settings retrieval");
        trace!("user_id: {user_id:?}");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/settings/imap"))?;
        let send = GmailSend::get(auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailImapGet {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailImapSettings>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail imap settings retrieved");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
