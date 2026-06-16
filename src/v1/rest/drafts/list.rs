//! List the Gmail drafts (`users.drafts.list`).

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use log::{debug, trace};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::drafts::GmailDraft,
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

/// Query parameters for listing drafts (`users.drafts.list`).
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct GmailDraftsListParams<'a> {
    pub q: Option<&'a str>,
    pub max_results: Option<u32>,
    pub page_token: Option<&'a str>,
    pub include_spam_trash: bool,
}

/// Response returned when listing drafts (`users.drafts.list`).
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailDraftsListResponse {
    #[serde(default)]
    pub drafts: Vec<GmailDraft>,
    #[serde(default)]
    pub next_page_token: Option<String>,
    #[serde(default)]
    pub result_size_estimate: Option<u64>,
}

pub struct GmailDraftsList {
    send: GmailSend<GmailDraftsListResponse>,
}

impl GmailDraftsList {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        params: &GmailDraftsListParams,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail drafts listing");
        trace!("params: {params:?}");

        let mut url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/drafts"))?;

        {
            let mut query = url.query_pairs_mut();

            if let Some(q) = params.q.filter(|q| !q.trim().is_empty()) {
                query.append_pair("q", q);
            }

            if let Some(max_results) = params.max_results {
                query.append_pair("maxResults", &max_results.min(500).to_string());
            }

            if let Some(page_token) = params.page_token {
                query.append_pair("pageToken", page_token);
            }

            if params.include_spam_trash {
                query.append_pair("includeSpamTrash", "true");
            }
        }

        let send = GmailSend::get(http_auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailDraftsList {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailDraftsListResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail drafts listed");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
