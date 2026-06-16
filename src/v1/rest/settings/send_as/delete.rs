//! Delete a Gmail send-as alias (`users.settings.sendAs.delete`).

use alloc::format;

use log::{debug, trace};
use secrecy::SecretString;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::send::{GMAIL_API_BASE, GmailNoResponse, GmailSend, GmailSendError, GmailSendOutput},
};

pub struct GmailSendAsDelete {
    send: GmailSend<GmailNoResponse>,
}

impl GmailSendAsDelete {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        send_as_email: &str,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail send-as alias deletion");
        trace!("send_as_email: {send_as_email:?}");

        let url = Url::parse(GMAIL_API_BASE)?
            .join(&format!("users/{user_id}/settings/sendAs/{send_as_email}"))?;
        let send = GmailSend::delete(http_auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailSendAsDelete {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailNoResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail send-as alias deleted");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
