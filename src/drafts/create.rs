use core::fmt;

use alloc::{format, string::String};

use log::trace;
use secrecy::SecretString;
use serde::Serialize;
use url::Url;

use crate::{
    coroutine::*,
    drafts::GmailDraft,
    gmail_try,
    messages::encode_raw,
    send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GmailDraftCreateRequest {
    message: DraftMessage,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DraftMessage {
    raw: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    thread_id: Option<String>,
}

pub struct GmailDraftCreate {
    state: State,
}

impl GmailDraftCreate {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        rfc5322: &[u8],
        thread_id: Option<&str>,
    ) -> Result<Self, GmailSendError> {
        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/drafts"))?;
        let body = GmailDraftCreateRequest {
            message: DraftMessage {
                raw: encode_raw(rfc5322),
                thread_id: thread_id.map(String::from),
            },
        };

        Ok(Self {
            state: State::Send(GmailSend::post_json(http_auth, url, &body)?),
        })
    }
}

impl GmailCoroutine for GmailDraftCreate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailDraft>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        trace!("draft-create: {}", self.state);
        match &mut self.state {
            State::Send(send) => {
                let out = gmail_try!(send, arg);
                GmailCoroutineState::Complete(Ok(out))
            }
        }
    }
}

enum State {
    Send(GmailSend<GmailDraft>),
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Send(_) => f.write_str("send"),
        }
    }
}
