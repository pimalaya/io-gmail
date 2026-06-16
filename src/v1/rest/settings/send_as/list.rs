//! List the Gmail send-as aliases (`users.settings.sendAs.list`).

use alloc::{format, vec::Vec};

use log::{debug, trace};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::send_as::GmailSendAs,
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

/// Response wrapping the send-as aliases of a Gmail account.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailSendAsListResponse {
    #[serde(default)]
    pub send_as: Vec<GmailSendAs>,
}

pub struct GmailSendAsList {
    send: GmailSend<GmailSendAsListResponse>,
}

impl GmailSendAsList {
    pub fn new(http_auth: &SecretString, user_id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail send-as aliases listing");
        trace!("user_id: {user_id:?}");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/settings/sendAs"))?;
        let send = GmailSend::get(http_auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailSendAsList {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailSendAsListResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail send-as aliases listed");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
