//! Gmail user profile (`users.getProfile`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users/getProfile>

use core::fmt;

use alloc::{format, string::String};

use log::trace;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
};

#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailProfile {
    pub email_address: String,
    #[serde(default)]
    pub messages_total: Option<u64>,
    #[serde(default)]
    pub threads_total: Option<u64>,
    #[serde(default)]
    pub history_id: Option<String>,
}

pub struct GmailProfileGet {
    state: State,
}

impl GmailProfileGet {
    pub fn new(http_auth: &SecretString, user_id: &str) -> Result<Self, GmailSendError> {
        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/profile"))?;

        Ok(Self {
            state: State::Send(GmailSend::get(http_auth, url)),
        })
    }
}

impl GmailCoroutine for GmailProfileGet {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailProfile>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        trace!("profile-get: {}", self.state);
        match &mut self.state {
            State::Send(send) => {
                let out = gmail_try!(send, arg);
                GmailCoroutineState::Complete(Ok(out))
            }
        }
    }
}

enum State {
    Send(GmailSend<GmailProfile>),
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Send(_) => f.write_str("send"),
        }
    }
}
