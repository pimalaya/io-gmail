use core::fmt;

use alloc::{format, string::String};

use log::trace;
use secrecy::SecretString;
use serde::Serialize;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::rest::messages::{GmailMessage, encode_raw},
    v1::send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GmailMessageInsertRequest<'a> {
    raw: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    label_ids: Option<&'a [String]>,
}

pub struct GmailMessageInsert {
    state: State,
}

impl GmailMessageInsert {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        rfc5322: &[u8],
        label_ids: &[String],
    ) -> Result<Self, GmailSendError> {
        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/messages"))?;
        let body = GmailMessageInsertRequest {
            raw: encode_raw(rfc5322),
            label_ids: if label_ids.is_empty() {
                None
            } else {
                Some(label_ids)
            },
        };

        Ok(Self {
            state: State::Send(GmailSend::post_json(http_auth, url, &body)?),
        })
    }
}

impl GmailCoroutine for GmailMessageInsert {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailMessage>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        trace!("message-insert: {}", self.state);
        match &mut self.state {
            State::Send(send) => {
                let out = gmail_try!(send, arg);
                GmailCoroutineState::Complete(Ok(out))
            }
        }
    }
}

enum State {
    Send(GmailSend<GmailMessage>),
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Send(_) => f.write_str("send"),
        }
    }
}
