use core::fmt;

use alloc::{format, string::String};

use log::trace;
use secrecy::SecretString;
use serde::Serialize;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::rest::labels::GmailLabel,
    v1::send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GmailLabelPatchRequest<'a> {
    name: &'a str,
    label_list_visibility: &'static str,
    message_list_visibility: &'static str,
}

pub struct GmailLabelPatch {
    state: State,
}

impl GmailLabelPatch {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        id: &str,
        name: &str,
    ) -> Result<Self, GmailSendError> {
        if name.trim().is_empty() {
            return Err(GmailSendError::InvalidRequest(String::from(
                "Label name cannot be empty",
            )));
        }

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/labels/{id}"))?;
        let body = GmailLabelPatchRequest {
            name,
            label_list_visibility: "labelShow",
            message_list_visibility: "show",
        };

        Ok(Self {
            state: State::Send(GmailSend::patch_json(http_auth, url, &body)?),
        })
    }
}

impl GmailCoroutine for GmailLabelPatch {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailLabel>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        trace!("label-patch: {}", self.state);
        match &mut self.state {
            State::Send(send) => {
                let out = gmail_try!(send, arg);
                GmailCoroutineState::Complete(Ok(out))
            }
        }
    }
}

enum State {
    Send(GmailSend<GmailLabel>),
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Send(_) => f.write_str("send"),
        }
    }
}
