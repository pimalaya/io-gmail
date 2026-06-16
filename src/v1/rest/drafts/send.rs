use core::fmt;

use alloc::{format, string::String};

use log::trace;
use secrecy::SecretString;
use serde::Serialize;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::rest::messages::GmailMessageId,
    v1::send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
};

#[derive(Debug, Serialize)]
struct GmailDraftSendRequest {
    id: String,
}

pub struct GmailDraftSend {
    state: State,
}

impl GmailDraftSend {
    pub fn new(http_auth: &SecretString, user_id: &str, id: &str) -> Result<Self, GmailSendError> {
        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/drafts/send"))?;
        let body = GmailDraftSendRequest {
            id: String::from(id),
        };

        Ok(Self {
            state: State::Send(GmailSend::post_json(http_auth, url, &body)?),
        })
    }
}

impl GmailCoroutine for GmailDraftSend {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailMessageId>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        trace!("draft-send: {}", self.state);
        match &mut self.state {
            State::Send(send) => {
                let out = gmail_try!(send, arg);
                GmailCoroutineState::Complete(Ok(out))
            }
        }
    }
}

enum State {
    Send(GmailSend<GmailMessageId>),
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Send(_) => f.write_str("send"),
        }
    }
}
