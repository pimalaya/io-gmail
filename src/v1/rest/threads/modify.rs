//! Modify Gmail thread labels (`users.threads.modify`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.threads/modify>

use alloc::{format, string::String};

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use serde::Serialize;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::rest::threads::GmailThread,
    v1::send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GmailThreadModifyRequest<'a> {
    add_label_ids: &'a [String],
    remove_label_ids: &'a [String],
}

/// Gmail REST thread label modification, wrapping the updated `GmailThread`.
pub struct GmailThreadModify {
    send: GmailSend<GmailThread>,
}

impl GmailThreadModify {
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        id: &str,
        add_label_ids: &[String],
        remove_label_ids: &[String],
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail thread modification");
        trace!("add_label_ids: {add_label_ids:?}");
        trace!("remove_label_ids: {remove_label_ids:?}");

        if add_label_ids.is_empty() && remove_label_ids.is_empty() {
            return Err(GmailSendError::InvalidRequest(String::from(
                "Modify requires at least one label update",
            )));
        }

        let url =
            Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/threads/{id}/modify"))?;
        let body = GmailThreadModifyRequest {
            add_label_ids,
            remove_label_ids,
        };
        let send = GmailSend::post_json(auth, url, &body)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailThreadModify {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailThread>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail thread modified");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
