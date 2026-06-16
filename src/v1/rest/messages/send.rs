use alloc::format;

use log::{debug, trace};
use secrecy::SecretString;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::rest::messages::{GmailMessage, GmailMessageId},
    v1::send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
};

/// Gmail REST message send, wrapping the resulting `GmailMessageId`.
pub struct GmailMessageSend {
    send: GmailSend<GmailMessageId>,
}

impl GmailMessageSend {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        message: &GmailMessage,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail message send");
        trace!("message: {message:?}");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/messages/send"))?;
        let send = GmailSend::post_json(http_auth, url, message)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailMessageSend {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailMessageId>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail message sent");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
