//! Update a Gmail draft (`users.drafts.update`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.drafts/update>

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

/// Gmail REST draft update, wrapping the updated `GmailDraft`.
pub struct GmailDraftUpdate {
    send: GmailSend<GmailDraft>,
}

impl GmailDraftUpdate {
    /// Builds the `users.drafts.update` request wrapping the given draft,
    /// addressed by its id.
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        draft: &GmailDraft,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail draft update");
        trace!("draft: {draft:?}");

        let id = &draft.id;

        let url = gmail_api_url(&["users", user_id, "drafts", id])?;
        let send = GmailSend::put_json(auth, url, draft)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailDraftUpdate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailDraft>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("draft updated");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
