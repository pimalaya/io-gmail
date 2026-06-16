use alloc::{format, vec::Vec};

use log::trace;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::forwarding_addresses::GmailForwardingAddress,
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

/// Response wrapping the forwarding addresses of a Gmail account.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailForwardingAddressesListResponse {
    #[serde(default)]
    pub forwarding_addresses: Vec<GmailForwardingAddress>,
}

pub struct GmailForwardingAddressesList {
    send: GmailSend<GmailForwardingAddressesListResponse>,
}

impl GmailForwardingAddressesList {
    pub fn new(http_auth: &SecretString, user_id: &str) -> Result<Self, GmailSendError> {
        trace!("prepare gmail forwarding addresses listing");

        let url = Url::parse(GMAIL_API_BASE)?
            .join(&format!("users/{user_id}/settings/forwardingAddresses"))?;
        let send = GmailSend::get(http_auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailForwardingAddressesList {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailForwardingAddressesListResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        trace!("gmail forwarding addresses listed: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
