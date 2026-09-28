//! Modify Gmail message labels (`users.messages.modify`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.messages/modify>

use alloc::{format, string::String};

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use serde::Serialize;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::messages::GmailMessage,
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GmailMessageModifyRequest<'a> {
    add_label_ids: &'a [String],
    remove_label_ids: &'a [String],
}

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

        if add_label_ids.is_empty() && remove_label_ids.is_empty() {
            return Err(GmailSendError::InvalidRequest(String::from(
                "Modify requires at least one label update",
            )));
        }

        let url =
            Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/messages/{id}/modify"))?;
        let body = GmailMessageModifyRequest {
            add_label_ids,
            remove_label_ids,
        };
        let send = GmailSend::post_json(auth, url, &body)?;

        Ok(Self { send })
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
