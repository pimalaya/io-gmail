use core::fmt;

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use log::trace;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::rest::threads::GmailThreadSummary,
    v1::send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
};

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

pub struct GmailThreadsList {
    state: State,
}

impl GmailThreadsList {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        q: Option<&str>,
        label_ids: &[String],
        max_results: Option<u32>,
        page_token: Option<&str>,
        include_spam_trash: bool,
    ) -> Result<Self, GmailSendError> {
        let mut url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/threads"))?;

        {
            let mut query = url.query_pairs_mut();

            if let Some(q) = q.filter(|q| !q.trim().is_empty()) {
                query.append_pair("q", q);
            }

            for label_id in label_ids {
                query.append_pair("labelIds", label_id);
            }

            if let Some(max_results) = max_results {
                query.append_pair("maxResults", &max_results.min(500).to_string());
            }

            if let Some(page_token) = page_token {
                query.append_pair("pageToken", page_token);
            }

            if include_spam_trash {
                query.append_pair("includeSpamTrash", "true");
            }
        }

        Ok(Self {
            state: State::Send(GmailSend::get(http_auth, url)),
        })
    }
}

impl GmailCoroutine for GmailThreadsList {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailThreadsListResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        trace!("threads-list: {}", self.state);
        match &mut self.state {
            State::Send(send) => {
                let out = gmail_try!(send, arg);
                GmailCoroutineState::Complete(Ok(out))
            }
        }
    }
}

enum State {
    Send(GmailSend<GmailThreadsListResponse>),
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Send(_) => f.write_str("send"),
        }
    }
}
