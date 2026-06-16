use alloc::{format, vec::Vec};

use log::trace;
use secrecy::SecretString;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::send::{GMAIL_API_BASE, GmailNoResponse, GmailSend, GmailSendError, GmailSendOutput},
};

pub struct GmailSendAsVerify {
    send: GmailSend<GmailNoResponse>,
}

impl GmailSendAsVerify {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        send_as_email: &str,
    ) -> Result<Self, GmailSendError> {
        trace!("prepare gmail send-as alias {send_as_email} verification");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!(
            "users/{user_id}/settings/sendAs/{send_as_email}/verify"
        ))?;
        let send = GmailSend::with_method(http_auth, "POST", url, None, Vec::new());

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailSendAsVerify {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailNoResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        trace!("gmail send-as alias verification requested: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
