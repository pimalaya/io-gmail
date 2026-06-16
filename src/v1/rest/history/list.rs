//! List the Gmail history records (`users.history.list`).

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use log::{debug, trace};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use serde_variant::to_variant_name;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::history::{GmailHistory, GmailHistoryType},
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

/// Query parameters for listing history records (`users.history.list`).
#[derive(Debug, Clone, Default, Eq, PartialEq)]
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
        http_auth: &SecretString,
        user_id: &str,
        params: &GmailHistoryListParams,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail history listing");
        trace!("params: {params:?}");

        let mut url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/history"))?;

        {
            let mut query = url.query_pairs_mut();

            query.append_pair("startHistoryId", params.start_history_id);

            if let Some(max_results) = params.max_results {
                query.append_pair("maxResults", &max_results.min(500).to_string());
            }

            if let Some(page_token) = params.page_token {
                query.append_pair("pageToken", page_token);
            }

            if let Some(label_id) = params.label_id {
                query.append_pair("labelId", label_id);
            }

            for history_type in params.history_types {
                let value = to_variant_name(history_type).unwrap_or_default();
                query.append_pair("historyTypes", value);
            }
        }

        let send = GmailSend::get(http_auth, url);

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
