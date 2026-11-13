//! Create a Gmail draft (`users.drafts.create`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.drafts/create>

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::drafts::GmailDraft,
        send::{GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
    },
};

/// Gmail REST draft creation, wrapping the created `GmailDraft`.
pub struct GmailDraftCreate {
    send: GmailSend<GmailDraft>,
}

impl GmailDraftCreate {
    /// Builds the `users.drafts.create` request wrapping the given draft.
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        draft: &GmailDraft,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail draft creation");
        trace!("draft: {draft:?}");

        let url = gmail_api_url(&["users", user_id, "drafts"])?;
        let send = GmailSend::post_json(auth, url, draft)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailDraftCreate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailDraft>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("draft created");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
