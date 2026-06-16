use alloc::{format, string::String};

use log::trace;
use secrecy::SecretString;
use serde::Serialize;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::rest::messages::GmailMessage,
    v1::send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
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
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        id: &str,
        add_label_ids: &[String],
        remove_label_ids: &[String],
    ) -> Result<Self, GmailSendError> {
        trace!("prepare gmail message {id} modification");

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
        let send = GmailSend::post_json(http_auth, url, &body)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailMessageModify {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailMessage>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        trace!("gmail message modified: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
