//! Verify a Gmail send-as alias (`users.settings.sendAs.verify`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.sendAs/verify>

use alloc::{format, vec::Vec};

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
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
        auth: &HttpAuthBearer,
        user_id: &str,
        send_as_email: &str,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail send-as alias verification");
        trace!("send_as_email: {send_as_email:?}");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!(
            "users/{user_id}/settings/sendAs/{send_as_email}/verify"
        ))?;
        let send = GmailSend::with_method(auth, "POST", url, None, Vec::new());

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailSendAsVerify {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailNoResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail send-as alias verification requested");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
