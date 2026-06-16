use alloc::{format, vec::Vec};

use log::trace;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::filters::GmailFilter,
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

/// Response wrapping the filters of a Gmail account.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailFiltersListResponse {
    #[serde(default)]
    pub filter: Vec<GmailFilter>,
}

pub struct GmailFiltersList {
    send: GmailSend<GmailFiltersListResponse>,
}

impl GmailFiltersList {
    pub fn new(http_auth: &SecretString, user_id: &str) -> Result<Self, GmailSendError> {
        trace!("prepare gmail filters listing");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/settings/filters"))?;
        let send = GmailSend::get(http_auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailFiltersList {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailFiltersListResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        trace!("gmail filters listed: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
