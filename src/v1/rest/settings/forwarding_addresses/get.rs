//! Get a Gmail forwarding address (`users.settings.forwardingAddresses.get`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.forwardingAddresses/get>

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

/// I/O-free coroutine getting a forwarding address of a Gmail account
/// (`users.settings.forwardingAddresses.get`).
pub struct GmailForwardingAddressGet {
    send: GmailSend<GmailForwardingAddress>,
}

impl GmailForwardingAddressGet {
    /// Builds the `users.settings.forwardingAddresses.get` request for
    /// the given forwarding email.
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        forwarding_email: &str,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail forwarding address retrieval");
        trace!("forwarding_email: {forwarding_email:?}");

        let url = gmail_api_url(&[
            "users",
            user_id,
            "settings",
            "forwardingAddresses",
            forwarding_email,
        ])?;
        let send = GmailSend::get(auth, url)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailForwardingAddressGet {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailForwardingAddress>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("forwarding address retrieved");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
