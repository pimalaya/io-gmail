//! Trash a Gmail message (`users.messages.trash`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.messages/trash>

use alloc::vec::Vec;

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::messages::GmailMessage,
        send::{GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
    },
};

/// Gmail REST message trashing, wrapping the updated `GmailMessage`.
pub struct GmailMessageTrash {
    send: GmailSend<GmailMessage>,
}

impl GmailMessageTrash {
    /// Builds the `users.messages.trash` request for the given message id.
    pub fn new(auth: &HttpAuthBearer, user_id: &str, id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail message trashing");
        trace!("id: {id:?}");

        let url = Self::url(user_id, id)?;
        let send = GmailSend::with_method(auth, "POST", url, None, Vec::new())?;

        Ok(Self { send })
    }

    /// Builds the `users.messages.trash` URL, shared with the batched
    /// form of the same call
    /// ([`GmailBatchRequest::message_trash`](crate::v1::batch::GmailBatchRequest::message_trash)).
    pub fn url(user_id: &str, id: &str) -> Result<Url, GmailSendError> {
        gmail_api_url(&["users", user_id, "messages", id, "trash"])
    }
}

impl GmailCoroutine for GmailMessageTrash {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailMessage>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("message trashed");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
