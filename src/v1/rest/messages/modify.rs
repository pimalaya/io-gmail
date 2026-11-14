//! Modify Gmail message labels (`users.messages.modify`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.messages/modify>

use alloc::string::String;

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use serde::Serialize;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::messages::GmailMessage,
        send::{GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
    },
};

/// Gmail REST message label modification, wrapping the updated `GmailMessage`.
pub struct GmailMessageModify {
    send: GmailSend<GmailMessage>,
}

impl GmailMessageModify {
    /// Builds the `users.messages.modify` request adding and removing
    /// the given label ids on the given message.
    ///
    /// Fails with [`GmailSendError::InvalidRequest`] when both label
    /// lists are empty.
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        id: &str,
        add_label_ids: &[String],
        remove_label_ids: &[String],
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail message modification");
        trace!("add_label_ids: {add_label_ids:?}");
        trace!("remove_label_ids: {remove_label_ids:?}");

        let url = Self::url(user_id, id)?;
        let body = GmailMessageModifyRequest::new(add_label_ids, remove_label_ids)?;
        let send = GmailSend::post_json(auth, url, &body)?;

        Ok(Self { send })
    }

    /// Builds the `users.messages.modify` URL, shared with the batched
    /// form of the same call
    /// ([`GmailBatchRequest::message_modify`](crate::v1::batch::GmailBatchRequest::message_modify)).
    pub fn url(user_id: &str, id: &str) -> Result<Url, GmailSendError> {
        gmail_api_url(&["users", user_id, "messages", id, "modify"])
    }
}

impl GmailCoroutine for GmailMessageModify {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailMessage>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("message modified");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}

/// The JSON body of `users.messages.modify`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GmailMessageModifyRequest<'a> {
    add_label_ids: &'a [String],
    remove_label_ids: &'a [String],
}

impl<'a> GmailMessageModifyRequest<'a> {
    /// Builds the body, refusing two empty label lists.
    pub(crate) fn new(
        add_label_ids: &'a [String],
        remove_label_ids: &'a [String],
    ) -> Result<Self, GmailSendError> {
        if add_label_ids.is_empty() && remove_label_ids.is_empty() {
            return Err(GmailSendError::InvalidRequest(String::from(
                "Modify requires at least one label update",
            )));
        }

        Ok(Self {
            add_label_ids,
            remove_label_ids,
        })
    }
}
