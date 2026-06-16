use alloc::{format, vec::Vec};

use log::trace;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::delegates::GmailDelegate,
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

/// Response wrapping the delegates of a Gmail account.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailDelegatesListResponse {
    #[serde(default)]
    pub delegates: Vec<GmailDelegate>,
}

pub struct GmailDelegatesList {
    send: GmailSend<GmailDelegatesListResponse>,
}

impl GmailDelegatesList {
    pub fn new(http_auth: &SecretString, user_id: &str) -> Result<Self, GmailSendError> {
        trace!("prepare gmail delegates listing");

        let url =
            Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/settings/delegates"))?;
        let send = GmailSend::get(http_auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailDelegatesList {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailDelegatesListResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        trace!("gmail delegates listed: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
