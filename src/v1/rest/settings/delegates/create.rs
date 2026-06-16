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

pub struct GmailDelegateCreate {
    send: GmailSend<GmailDelegate>,
}

impl GmailDelegateCreate {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        delegate: &GmailDelegate,
    ) -> Result<Self, GmailSendError> {
        trace!("prepare gmail delegate creation");

        let url =
            Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/settings/delegates"))?;
        let send = GmailSend::post_json(http_auth, url, delegate)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailDelegateCreate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailDelegate>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        trace!("gmail delegate created: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
