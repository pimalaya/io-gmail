use alloc::format;

use log::trace;
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

pub struct GmailSendAsGet {
    send: GmailSend<GmailSendAs>,
}

impl GmailSendAsGet {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        send_as_email: &str,
    ) -> Result<Self, GmailSendError> {
        trace!("prepare gmail send-as alias {send_as_email} retrieval");

        let url = Url::parse(GMAIL_API_BASE)?
            .join(&format!("users/{user_id}/settings/sendAs/{send_as_email}"))?;
        let send = GmailSend::get(http_auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailSendAsGet {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailSendAs>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        trace!("gmail send-as alias retrieved: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
