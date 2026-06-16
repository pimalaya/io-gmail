use alloc::format;

use log::trace;
use secrecy::SecretString;
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
        http_auth: &SecretString,
        user_id: &str,
        id: &str,
        format: GmailMessageFormat,
    ) -> Result<Self, GmailSendError> {
        trace!("prepare gmail draft {id} retrieval");

        let mut url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/drafts/{id}"))?;

        {
            let mut query = url.query_pairs_mut();
            query.append_pair("format", format.as_str());
        }

        let send = GmailSend::get(http_auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailDraftGet {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailDraft>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        trace!("gmail draft retrieved: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
