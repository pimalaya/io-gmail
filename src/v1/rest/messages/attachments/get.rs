use alloc::format;

use log::{debug, trace};
use secrecy::SecretString;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::rest::messages::GmailMessagePartBody,
    v1::send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
};

/// Gmail REST attachment retrieval, wrapping a `GmailMessagePartBody`.
pub struct GmailAttachmentGet {
    send: GmailSend<GmailMessagePartBody>,
}

impl GmailAttachmentGet {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        message_id: &str,
        id: &str,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail attachment retrieval");
        trace!("message_id: {message_id:?}");
        trace!("id: {id:?}");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!(
            "users/{user_id}/messages/{message_id}/attachments/{id}"
        ))?;
        let send = GmailSend::get(http_auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailAttachmentGet {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailMessagePartBody>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail attachment retrieved");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
