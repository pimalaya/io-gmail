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
    v1::rest::history::GmailHistory,
    v1::send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
};

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
    state: State,
}

impl GmailHistoryList {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        start_history_id: &str,
        label_id: Option<&str>,
        history_types: &[&str],
        max_results: Option<u32>,
        page_token: Option<&str>,
    ) -> Result<Self, GmailSendError> {
        let mut url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/history"))?;

        {
            let mut query = url.query_pairs_mut();

            query.append_pair("startHistoryId", start_history_id);

            if let Some(max_results) = max_results {
                query.append_pair("maxResults", &max_results.min(500).to_string());
            }

            if let Some(page_token) = page_token {
                query.append_pair("pageToken", page_token);
            }

            if let Some(label_id) = label_id {
                query.append_pair("labelId", label_id);
            }

            for history_type in history_types {
                query.append_pair("historyTypes", history_type);
            }
        }

        Ok(Self {
            state: State::Send(GmailSend::get(http_auth, url)),
        })
    }
}

impl GmailCoroutine for GmailHistoryList {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailHistoryListResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        trace!("history-list: {}", self.state);
        match &mut self.state {
            State::Send(send) => {
                let out = gmail_try!(send, arg);
                GmailCoroutineState::Complete(Ok(out))
            }
        }
    }
}

enum State {
    Send(GmailSend<GmailHistoryListResponse>),
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Send(_) => f.write_str("send"),
        }
    }
}
