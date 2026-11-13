//! List the Gmail forwarding addresses
//! (`users.settings.forwardingAddresses.list`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.forwardingAddresses/list>

use alloc::vec::Vec;

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use serde::{Deserialize, Serialize};

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::forwarding_addresses::GmailForwardingAddress,
        send::{GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
    },
};

/// Response wrapping the forwarding addresses of a Gmail account.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GmailForwardingAddressesListResponse {
    /// Forwarding addresses of the Gmail account.
    #[serde(default)]
    pub forwarding_addresses: Vec<GmailForwardingAddress>,
}

/// I/O-free coroutine listing the forwarding addresses of a Gmail
/// account (`users.settings.forwardingAddresses.list`).
pub struct GmailForwardingAddressesList {
    send: GmailSend<GmailForwardingAddressesListResponse>,
}

impl GmailForwardingAddressesList {
    /// Builds the `users.settings.forwardingAddresses.list` request for
    /// the given user.
    pub fn new(auth: &HttpAuthBearer, user_id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail forwarding addresses listing");
        trace!("user_id: {user_id:?}");

        let url = gmail_api_url(&["users", user_id, "settings", "forwardingAddresses"])?;
        let send = GmailSend::get(auth, url)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailForwardingAddressesList {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailForwardingAddressesListResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("forwarding addresses listed");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
