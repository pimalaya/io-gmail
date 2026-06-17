//! List the Gmail drafts (`users.drafts.list`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.drafts/list>

use alloc::{format, string::String, vec::Vec};

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        query::to_query_pairs,
        rest::drafts::GmailDraft,
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

/// Query parameters for listing drafts (`users.drafts.list`).
#[derive(Debug, Clone, Default, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailDraftsListParams<'a> {
    pub q: Option<&'a str>,
    pub max_results: Option<u32>,
    pub page_token: Option<&'a str>,
    #[serde(skip_serializing_if = "crate::v1::query::is_false")]
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
        auth: &HttpAuthBearer,
        user_id: &str,
        params: &GmailDraftsListParams,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail drafts listing");
        trace!("params: {params:?}");

        let mut url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/drafts"))?;
        url.query_pairs_mut().extend_pairs(to_query_pairs(params));

        let send = GmailSend::get(auth, url);

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
