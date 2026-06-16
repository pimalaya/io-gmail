use alloc::{format, string::String};

use log::trace;
use secrecy::SecretString;
use serde::Serialize;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::send::{GMAIL_API_BASE, GmailNoResponse, GmailSend, GmailSendError, GmailSendOutput},
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GmailMessageBatchModifyRequest<'a> {
    ids: &'a [String],
    add_label_ids: &'a [String],
    remove_label_ids: &'a [String],
}

/// Gmail REST batch message label modification, yielding no response body.
pub struct GmailMessageBatchModify {
    send: GmailSend<GmailNoResponse>,
}

impl GmailMessageBatchModify {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        ids: &[String],
        add_label_ids: &[String],
        remove_label_ids: &[String],
    ) -> Result<Self, GmailSendError> {
        trace!("prepare gmail messages batch modification");

        let url =
            Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/messages/batchModify"))?;
        let body = GmailMessageBatchModifyRequest {
            ids,
            add_label_ids,
            remove_label_ids,
        };
        let send = GmailSend::post_json(http_auth, url, &body)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailMessageBatchModify {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailNoResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        trace!("gmail messages batch modified: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
