//! Delete a Gmail forwarding address
//! (`users.settings.forwardingAddresses.delete`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.forwardingAddresses/delete>

use alloc::format;

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
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
        auth: &HttpAuthBearer,
        user_id: &str,
        forwarding_email: &str,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail forwarding address deletion");
        trace!("forwarding_email: {forwarding_email:?}");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!(
            "users/{user_id}/settings/forwardingAddresses/{forwarding_email}"
        ))?;
        let send = GmailSend::delete(auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailForwardingAddressDelete {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailNoResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail forwarding address deleted");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
