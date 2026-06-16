use alloc::format;

use log::trace;
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

pub struct GmailForwardingAddressGet {
    send: GmailSend<GmailForwardingAddress>,
}

impl GmailForwardingAddressGet {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        forwarding_email: &str,
    ) -> Result<Self, GmailSendError> {
        trace!("prepare gmail forwarding address {forwarding_email} retrieval");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!(
            "users/{user_id}/settings/forwardingAddresses/{forwarding_email}"
        ))?;
        let send = GmailSend::get(http_auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailForwardingAddressGet {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailForwardingAddress>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        trace!("gmail forwarding address retrieved: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
