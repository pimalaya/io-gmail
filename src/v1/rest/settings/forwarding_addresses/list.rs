//! List the Gmail forwarding addresses
//! (`users.settings.forwardingAddresses.list`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.forwardingAddresses/list>

use alloc::{format, vec::Vec};

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
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
    pub fn new(auth: &HttpAuthBearer, user_id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail forwarding addresses listing");
        trace!("user_id: {user_id:?}");

        let url = Url::parse(GMAIL_API_BASE)?
            .join(&format!("users/{user_id}/settings/forwardingAddresses"))?;
        let send = GmailSend::get(auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailForwardingAddressesList {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailForwardingAddressesListResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail forwarding addresses listed");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
