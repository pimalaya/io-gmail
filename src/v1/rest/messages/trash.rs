//! Trash a Gmail message (`users.messages.trash`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.messages/trash>

use alloc::{format, vec::Vec};

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
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
    pub fn new(auth: &HttpAuthBearer, user_id: &str, id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail message trashing");
        trace!("id: {id:?}");

        let url =
            Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/messages/{id}/trash"))?;
        let send = GmailSend::with_method(auth, "POST", url, None, Vec::new());

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailMessageTrash {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailMessage>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail message trashed");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
