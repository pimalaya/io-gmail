//! List the Gmail filters (`users.settings.filters.list`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.filters/list>

use alloc::{format, vec::Vec};

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::filters::GmailFilter,
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

/// Response wrapping the filters of a Gmail account.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailFiltersListResponse {
    #[serde(default)]
    pub filter: Vec<GmailFilter>,
}

pub struct GmailFiltersList {
    send: GmailSend<GmailFiltersListResponse>,
}

impl GmailFiltersList {
    pub fn new(auth: &HttpAuthBearer, user_id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail filters listing");
        trace!("user_id: {user_id:?}");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/settings/filters"))?;
        let send = GmailSend::get(auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailFiltersList {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailFiltersListResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail filters listed");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
