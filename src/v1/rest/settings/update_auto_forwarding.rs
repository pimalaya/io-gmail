use alloc::format;

use log::trace;
use secrecy::SecretString;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::GmailAutoForwarding,
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

pub struct GmailAutoForwardingUpdate {
    send: GmailSend<GmailAutoForwarding>,
}

impl GmailAutoForwardingUpdate {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        settings: GmailAutoForwarding,
    ) -> Result<Self, GmailSendError> {
        trace!("prepare gmail auto-forwarding settings update");

        let url = Url::parse(GMAIL_API_BASE)?
            .join(&format!("users/{user_id}/settings/autoForwarding"))?;
        let send = GmailSend::put_json(http_auth, url, &settings)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailAutoForwardingUpdate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailAutoForwarding>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        trace!("gmail auto-forwarding settings updated: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
