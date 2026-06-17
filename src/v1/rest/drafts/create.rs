//! Create a Gmail draft (`users.drafts.create`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.drafts/create>

use alloc::format;

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
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
        auth: &HttpAuthBearer,
        user_id: &str,
        draft: &GmailDraft,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail draft creation");
        trace!("draft: {draft:?}");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/drafts"))?;
        let send = GmailSend::post_json(auth, url, draft)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailDraftCreate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailDraft>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail draft created");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
