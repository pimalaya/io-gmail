use core::fmt;

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

pub struct GmailMessageBatchModify {
    state: State,
}

impl GmailMessageBatchModify {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        ids: &[String],
        add_label_ids: &[String],
        remove_label_ids: &[String],
    ) -> Result<Self, GmailSendError> {
        let url =
            Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/messages/batchModify"))?;
        let body = GmailMessageBatchModifyRequest {
            ids,
            add_label_ids,
            remove_label_ids,
        };

        Ok(Self {
            state: State::Send(GmailSend::post_json(http_auth, url, &body)?),
        })
    }
}

impl GmailCoroutine for GmailMessageBatchModify {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailNoResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        trace!("message-batch-modify: {}", self.state);
        match &mut self.state {
            State::Send(send) => {
                let out = gmail_try!(send, arg);
                GmailCoroutineState::Complete(Ok(out))
            }
        }
    }
}

enum State {
    Send(GmailSend<GmailNoResponse>),
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Send(_) => f.write_str("send"),
        }
    }
}
