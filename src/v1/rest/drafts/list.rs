//! List the Gmail drafts (`users.drafts.list`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.drafts/list>

use alloc::{string::String, vec::Vec};

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use serde::{Deserialize, Serialize};

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        query::{is_false, to_query_pairs},
        rest::drafts::GmailDraft,
        send::{GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
    },
};

/// Query parameters for listing drafts (`users.drafts.list`).
#[derive(Debug, Clone, Default, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailDraftsListParams<'a> {
    /// Search query filtering drafts, using the Gmail search box syntax.
    pub q: Option<&'a str>,
    /// Maximum number of drafts to return per page.
    pub max_results: Option<u32>,
    /// Page token from a previous listing response.
    pub page_token: Option<&'a str>,
    /// Whether to include drafts from SPAM and TRASH.
    #[serde(skip_serializing_if = "is_false")]
    pub include_spam_trash: bool,
}

/// Response returned when listing drafts (`users.drafts.list`).
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailDraftsListResponse {
    /// Drafts of the current page.
    #[serde(default)]
    pub drafts: Vec<GmailDraft>,
    /// Token to fetch the next page, absent on the last page.
    #[serde(default)]
    pub next_page_token: Option<String>,
    /// Estimated total number of results.
    #[serde(default)]
    pub result_size_estimate: Option<u64>,
}

/// I/O-free coroutine listing Gmail drafts (`users.drafts.list`).
pub struct GmailDraftsList {
    send: GmailSend<GmailDraftsListResponse>,
}

impl GmailDraftsList {
    /// Builds the `users.drafts.list` request from the given query
    /// parameters; `user_id` is the mailbox owner (usually `me`).
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        params: &GmailDraftsListParams,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail drafts listing");
        trace!("params: {params:?}");

        let mut url = gmail_api_url(&["users", user_id, "drafts"])?;
        url.query_pairs_mut().extend_pairs(to_query_pairs(params));

        let send = GmailSend::get(auth, url)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailDraftsList {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailDraftsListResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("drafts listed");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
