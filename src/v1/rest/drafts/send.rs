//! Send a Gmail draft (`users.drafts.send`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.drafts/send>

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::{drafts::GmailDraft, messages::GmailMessageId},
        send::{GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
    },
};

/// Gmail REST draft send, wrapping the resulting `GmailMessageId`.
pub struct GmailDraftSend {
    send: GmailSend<GmailMessageId>,
}

impl GmailDraftSend {
    /// Builds the `users.drafts.send` request wrapping the given draft.
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        draft: &GmailDraft,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail draft send");
        trace!("draft: {draft:?}");

        let url = gmail_api_url(&["users", user_id, "drafts", "send"])?;
        let send = GmailSend::post_json(auth, url, draft)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailDraftSend {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailMessageId>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("draft sent");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
