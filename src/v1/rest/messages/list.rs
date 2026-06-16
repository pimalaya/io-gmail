//! List the Gmail messages (`users.messages.list`).

use alloc::{format, string::String, vec::Vec};

use log::{debug, trace};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        query::to_query_pairs,
        rest::messages::GmailMessageId,
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

/// Query parameters for listing messages (`users.messages.list`).
#[derive(Debug, Clone, Default, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailMessagesListParams<'a> {
    pub q: Option<&'a str>,
    pub label_ids: &'a [String],
    pub max_results: Option<u32>,
    pub page_token: Option<&'a str>,
    #[serde(skip_serializing_if = "crate::v1::query::is_false")]
    pub include_spam_trash: bool,
}

/// Gmail REST message listing response (one page of message ids).
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailMessagesListResponse {
    #[serde(default)]
    pub messages: Vec<GmailMessageId>,
    #[serde(default)]
    pub next_page_token: Option<String>,
    #[serde(default)]
    pub result_size_estimate: Option<u64>,
}

/// Gmail REST message listing, wrapping a page of message ids.
pub struct GmailMessagesList {
    send: GmailSend<GmailMessagesListResponse>,
}

impl GmailMessagesList {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        params: &GmailMessagesListParams,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail messages listing");
        trace!("params: {params:?}");

        let mut url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/messages"))?;
        url.query_pairs_mut().extend_pairs(to_query_pairs(params));

        let send = GmailSend::get(http_auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailMessagesList {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailMessagesListResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail messages listed");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
