use alloc::format;

use log::trace;
use secrecy::SecretString;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::send::{GMAIL_API_BASE, GmailNoResponse, GmailSend, GmailSendError, GmailSendOutput},
};

pub struct GmailForwardingAddressDelete {
    send: GmailSend<GmailNoResponse>,
}

impl GmailForwardingAddressDelete {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        forwarding_email: &str,
    ) -> Result<Self, GmailSendError> {
        trace!("prepare gmail forwarding address {forwarding_email} deletion");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!(
            "users/{user_id}/settings/forwardingAddresses/{forwarding_email}"
        ))?;
        let send = GmailSend::delete(http_auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailForwardingAddressDelete {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailNoResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        trace!("gmail forwarding address deleted: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
