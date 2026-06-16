//! Create a Gmail forwarding address
//! (`users.settings.forwardingAddresses.create`).

use alloc::format;

use log::{debug, trace};
use secrecy::SecretString;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::forwarding_addresses::GmailForwardingAddress,
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

pub struct GmailForwardingAddressCreate {
    send: GmailSend<GmailForwardingAddress>,
}

impl GmailForwardingAddressCreate {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        address: &GmailForwardingAddress,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail forwarding address creation");
        trace!("address: {address:?}");

        let url = Url::parse(GMAIL_API_BASE)?
            .join(&format!("users/{user_id}/settings/forwardingAddresses"))?;
        let send = GmailSend::post_json(http_auth, url, address)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailForwardingAddressCreate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailForwardingAddress>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail forwarding address created");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
