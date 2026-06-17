//! List the Gmail threads (`users.threads.list`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.threads/list>

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
        rest::threads::GmailThreadSummary,
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

/// Query parameters for listing threads (`users.threads.list`).
#[derive(Debug, Clone, Default, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailThreadsListParams<'a> {
    pub q: Option<&'a str>,
    pub label_ids: &'a [String],
    pub max_results: Option<u32>,
    pub page_token: Option<&'a str>,
    #[serde(skip_serializing_if = "crate::v1::query::is_false")]
    pub include_spam_trash: bool,
}

/// Gmail REST thread listing response (one page of thread summaries).
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailThreadsListResponse {
    #[serde(default)]
    pub threads: Vec<GmailThreadSummary>,
    #[serde(default)]
    pub next_page_token: Option<String>,
    #[serde(default)]
    pub result_size_estimate: Option<u64>,
}

/// Gmail REST thread listing, wrapping a page of thread summaries.
pub struct GmailThreadsList {
    send: GmailSend<GmailThreadsListResponse>,
}

impl GmailThreadsList {
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        params: &GmailThreadsListParams,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail threads listing");
        trace!("params: {params:?}");

        let mut url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/threads"))?;
        url.query_pairs_mut().extend_pairs(to_query_pairs(params));

        let send = GmailSend::get(auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailThreadsList {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailThreadsListResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail threads listed");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
