use alloc::format;

use log::trace;
use secrecy::SecretString;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::delegates::GmailDelegate,
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

pub struct GmailDelegateGet {
    send: GmailSend<GmailDelegate>,
}

impl GmailDelegateGet {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        delegate_email: &str,
    ) -> Result<Self, GmailSendError> {
        trace!("prepare gmail delegate {delegate_email} retrieval");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!(
            "users/{user_id}/settings/delegates/{delegate_email}"
        ))?;
        let send = GmailSend::get(http_auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailDelegateGet {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailDelegate>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        trace!("gmail delegate retrieved: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
