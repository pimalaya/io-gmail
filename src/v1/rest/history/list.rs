//! List the Gmail history records (`users.history.list`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.history/list>

use alloc::{string::String, vec::Vec};

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use serde::{Deserialize, Serialize};

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        query::to_query_pairs,
        rest::history::{GmailHistory, GmailHistoryType},
        send::{GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
    },
};

/// Query parameters for listing history records (`users.history.list`).
#[derive(Debug, Clone, Default, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailHistoryListParams<'a> {
    /// The history id to start listing changes after.
    pub start_history_id: &'a str,
    /// The label id to restrict history records to.
    pub label_id: Option<&'a str>,
    /// The history types to return.
    pub history_types: &'a [GmailHistoryType],
    /// The maximum number of history records to return per page.
    pub max_results: Option<u32>,
    /// The page token to retrieve a specific page of results.
    pub page_token: Option<&'a str>,
}

/// Response returned when listing history records (`users.history.list`).
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailHistoryListResponse {
    /// The list of history records.
    #[serde(default)]
    pub history: Vec<GmailHistory>,
    /// The page token to retrieve the next page of results.
    #[serde(default)]
    pub next_page_token: Option<String>,
    /// The id of the current history record of the mailbox.
    #[serde(default)]
    pub history_id: Option<String>,
}

/// I/O-free coroutine listing Gmail history records (`users.history.list`).
pub struct GmailHistoryList {
    send: GmailSend<GmailHistoryListResponse>,
}

impl GmailHistoryList {
    /// Builds the `users.history.list` request from the given
    /// [`GmailHistoryListParams`].
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        params: &GmailHistoryListParams,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail history listing");
        trace!("params: {params:?}");

        let mut url = gmail_api_url(&["users", user_id, "history"])?;
        url.query_pairs_mut().extend_pairs(to_query_pairs(params));

        let send = GmailSend::get(auth, url)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailHistoryList {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailHistoryListResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("history listed");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
