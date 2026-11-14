//! Delete a Gmail message (`users.messages.delete`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.messages/delete>

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::send::{GmailNoResponse, GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
};

/// Gmail REST message permanent deletion, yielding no response body.
pub struct GmailMessageDelete {
    send: GmailSend<GmailNoResponse>,
}

impl GmailMessageDelete {
    /// Builds the `users.messages.delete` request for the given
    /// message id.
    pub fn new(auth: &HttpAuthBearer, user_id: &str, id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail message deletion");
        trace!("id: {id:?}");

        let url = Self::url(user_id, id)?;
        let send = GmailSend::delete(auth, url)?;

        Ok(Self { send })
    }

    /// Builds the `users.messages.delete` URL, shared with the batched
    /// form of the same call
    /// ([`GmailBatchRequest::message_delete`](crate::v1::batch::GmailBatchRequest::message_delete)).
    pub fn url(user_id: &str, id: &str) -> Result<Url, GmailSendError> {
        gmail_api_url(&["users", user_id, "messages", id])
    }
}

impl GmailCoroutine for GmailMessageDelete {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailNoResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("message deleted");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
