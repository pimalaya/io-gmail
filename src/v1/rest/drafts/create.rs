use alloc::format;

use log::trace;
use secrecy::SecretString;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::drafts::GmailDraft,
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

/// Gmail REST draft creation, wrapping the created `GmailDraft`.
pub struct GmailDraftCreate {
    send: GmailSend<GmailDraft>,
}

impl GmailDraftCreate {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        draft: &GmailDraft,
    ) -> Result<Self, GmailSendError> {
        trace!("prepare gmail draft creation");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/drafts"))?;
        let send = GmailSend::post_json(http_auth, url, draft)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailDraftCreate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailDraft>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        trace!("gmail draft created: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
