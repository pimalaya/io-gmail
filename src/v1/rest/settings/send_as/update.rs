//! Update a Gmail send-as alias (`users.settings.sendAs.update`).

use alloc::format;

use log::{debug, trace};
use secrecy::SecretString;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::send_as::GmailSendAs,
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

pub struct GmailSendAsUpdate {
    send: GmailSend<GmailSendAs>,
}

impl GmailSendAsUpdate {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        send_as_email: &str,
        send_as: &GmailSendAs,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail send-as alias update");
        trace!("send_as: {send_as:?}");

        let url = Url::parse(GMAIL_API_BASE)?
            .join(&format!("users/{user_id}/settings/sendAs/{send_as_email}"))?;
        let send = GmailSend::put_json(http_auth, url, send_as)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailSendAsUpdate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailSendAs>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail send-as alias updated");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
