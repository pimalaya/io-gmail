use alloc::{format, vec::Vec};

use log::trace;
use secrecy::SecretString;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::rest::messages::GmailMessage,
    v1::send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
};

/// Gmail REST message trashing, wrapping the updated `GmailMessage`.
pub struct GmailMessageTrash {
    send: GmailSend<GmailMessage>,
}

impl GmailMessageTrash {
    pub fn new(http_auth: &SecretString, user_id: &str, id: &str) -> Result<Self, GmailSendError> {
        trace!("prepare gmail message {id} trashing");

        let url =
            Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/messages/{id}/trash"))?;
        let send = GmailSend::with_method(http_auth, "POST", url, None, Vec::new());

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailMessageTrash {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailMessage>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        trace!("gmail message trashed: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
