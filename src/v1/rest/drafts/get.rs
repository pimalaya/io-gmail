//! Get a Gmail draft (`users.drafts.get`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.drafts/get>

use alloc::format;

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use serde_variant::to_variant_name;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::{drafts::GmailDraft, messages::GmailMessageFormat},
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

/// Gmail REST draft retrieval, wrapping a `GmailDraft` response.
pub struct GmailDraftGet {
    send: GmailSend<GmailDraft>,
}

impl GmailDraftGet {
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        id: &str,
        format: GmailMessageFormat,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail draft retrieval");
        trace!("id: {id:?}");

        let mut url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/drafts/{id}"))?;

        {
            let mut query = url.query_pairs_mut();
            query.append_pair("format", to_variant_name(&format).unwrap_or_default());
        }

        let send = GmailSend::get(auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailDraftGet {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailDraft>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail draft retrieved");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
