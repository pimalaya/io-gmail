//! List the Gmail history records (`users.history.list`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.history/list>

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
        rest::history::{GmailHistory, GmailHistoryType},
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

/// Query parameters for listing history records (`users.history.list`).
#[derive(Debug, Clone, Default, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailHistoryListParams<'a> {
    pub start_history_id: &'a str,
    pub label_id: Option<&'a str>,
    pub history_types: &'a [GmailHistoryType],
    pub max_results: Option<u32>,
    pub page_token: Option<&'a str>,
}

/// Response returned when listing history records (`users.history.list`).
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailHistoryListResponse {
    #[serde(default)]
    pub history: Vec<GmailHistory>,
    #[serde(default)]
    pub next_page_token: Option<String>,
    #[serde(default)]
    pub history_id: Option<String>,
}

pub struct GmailHistoryList {
    send: GmailSend<GmailHistoryListResponse>,
}

impl GmailHistoryList {
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        params: &GmailHistoryListParams,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail history listing");
        trace!("params: {params:?}");

        let mut url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/history"))?;
        url.query_pairs_mut().extend_pairs(to_query_pairs(params));

        let send = GmailSend::get(auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailHistoryList {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailHistoryListResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail history listed");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
