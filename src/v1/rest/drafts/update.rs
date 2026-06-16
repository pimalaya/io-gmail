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

/// Gmail REST draft update, wrapping the updated `GmailDraft`.
pub struct GmailDraftUpdate {
    send: GmailSend<GmailDraft>,
}

impl GmailDraftUpdate {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        draft: &GmailDraft,
    ) -> Result<Self, GmailSendError> {
        let id = &draft.id;
        trace!("prepare gmail draft {id} update");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/drafts/{id}"))?;
        let send = GmailSend::put_json(http_auth, url, draft)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailDraftUpdate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailDraft>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        trace!("gmail draft updated: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
