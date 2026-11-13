//! List the Gmail messages (`users.messages.list`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.messages/list>

use alloc::{string::String, vec::Vec};

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use serde::{Deserialize, Serialize};

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        query::{is_false, to_query_pairs},
        rest::messages::GmailMessageId,
        send::{GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
    },
};

/// Query parameters for listing messages (`users.messages.list`).
#[derive(Debug, Clone, Default, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailMessagesListParams<'a> {
    /// The search query filtering the returned messages, using the
    /// Gmail search box syntax.
    pub q: Option<&'a str>,
    /// The label ids the returned messages must all carry.
    pub label_ids: &'a [String],
    /// The maximum number of messages to return per page.
    pub max_results: Option<u32>,
    /// The page token from a previous list response.
    pub page_token: Option<&'a str>,
    /// Whether to include messages from SPAM and TRASH in the results.
    #[serde(skip_serializing_if = "is_false")]
    pub include_spam_trash: bool,
}

/// Gmail REST message listing response (one page of message ids).
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailMessagesListResponse {
    /// The messages of the current page, each carrying only an id
    /// and a thread id.
    #[serde(default)]
    pub messages: Vec<GmailMessageId>,
    /// The token retrieving the next page of results, absent on the
    /// last page.
    #[serde(default)]
    pub next_page_token: Option<String>,
    /// The estimated total number of results.
    #[serde(default)]
    pub result_size_estimate: Option<u64>,
}

/// Gmail REST message listing, wrapping a page of message ids.
pub struct GmailMessagesList {
    send: GmailSend<GmailMessagesListResponse>,
}

impl GmailMessagesList {
    /// Builds the `users.messages.list` request from the given
    /// query parameters.
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        params: &GmailMessagesListParams,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail messages listing");
        trace!("params: {params:?}");

        let mut url = gmail_api_url(&["users", user_id, "messages"])?;
        url.query_pairs_mut().extend_pairs(to_query_pairs(params));

        let send = GmailSend::get(auth, url)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailMessagesList {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailMessagesListResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("messages listed");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
