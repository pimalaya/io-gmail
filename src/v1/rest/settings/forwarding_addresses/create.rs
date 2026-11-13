//! Create a Gmail forwarding address
//! (`users.settings.forwardingAddresses.create`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.forwardingAddresses/create>

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::forwarding_addresses::GmailForwardingAddress,
        send::{GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
    },
};

/// I/O-free coroutine creating a forwarding address on a Gmail account
/// (`users.settings.forwardingAddresses.create`).
pub struct GmailForwardingAddressCreate {
    send: GmailSend<GmailForwardingAddress>,
}

impl GmailForwardingAddressCreate {
    /// Builds the `users.settings.forwardingAddresses.create` request for
    /// the given forwarding address.
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        address: &GmailForwardingAddress,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail forwarding address creation");
        trace!("address: {address:?}");

        let url = gmail_api_url(&["users", user_id, "settings", "forwardingAddresses"])?;
        let send = GmailSend::post_json(auth, url, address)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailForwardingAddressCreate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailForwardingAddress>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("forwarding address created");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
