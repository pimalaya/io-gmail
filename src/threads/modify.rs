use core::fmt;

use alloc::{format, string::String};

use log::trace;
use secrecy::SecretString;
use serde::Serialize;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    threads::GmailThread,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GmailThreadModifyRequest<'a> {
    add_label_ids: &'a [String],
    remove_label_ids: &'a [String],
}

pub struct GmailThreadModify {
    state: State,
}

impl GmailThreadModify {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        id: &str,
        add_label_ids: &[String],
        remove_label_ids: &[String],
    ) -> Result<Self, GmailSendError> {
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

        Ok(Self {
            state: State::Send(GmailSend::post_json(http_auth, url, &body)?),
        })
    }
}

impl GmailCoroutine for GmailThreadModify {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailThread>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        trace!("thread-modify: {}", self.state);
        match &mut self.state {
            State::Send(send) => {
                let out = gmail_try!(send, arg);
                GmailCoroutineState::Complete(Ok(out))
            }
        }
    }
}

enum State {
    Send(GmailSend<GmailThread>),
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Send(_) => f.write_str("send"),
        }
    }
}
