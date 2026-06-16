use alloc::format;

use log::{debug, trace};
use secrecy::SecretString;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::{drafts::GmailDraft, messages::GmailMessageId},
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

/// Gmail REST draft send, wrapping the resulting `GmailMessageId`.
pub struct GmailDraftSend {
    send: GmailSend<GmailMessageId>,
}

impl GmailDraftSend {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        draft: &GmailDraft,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail draft send");
        trace!("draft: {draft:?}");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/drafts/send"))?;
        let send = GmailSend::post_json(http_auth, url, draft)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailDraftSend {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailMessageId>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail draft sent");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
