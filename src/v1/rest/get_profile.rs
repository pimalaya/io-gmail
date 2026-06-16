//! Gmail user profile (`users.getProfile`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users/getProfile>

use alloc::{format, string::String};

use log::trace;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
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
    send: GmailSend<GmailProfile>,
}

impl GmailProfileGet {
    pub fn new(http_auth: &SecretString, user_id: &str) -> Result<Self, GmailSendError> {
        trace!("prepare gmail profile retrieval");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/profile"))?;
        let send = GmailSend::get(http_auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailProfileGet {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailProfile>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        trace!("gmail profile retrieved: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
