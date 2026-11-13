//! List the Gmail filters (`users.settings.filters.list`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.filters/list>

use alloc::vec::Vec;

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use serde::{Deserialize, Serialize};

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::filters::GmailFilter,
        send::{GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
    },
};

/// Response wrapping the filters of a Gmail account.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GmailFiltersListResponse {
    /// Filters of the Gmail account.
    #[serde(default)]
    pub filter: Vec<GmailFilter>,
}

/// I/O-free coroutine listing the filters of a Gmail account
/// (`users.settings.filters.list`).
pub struct GmailFiltersList {
    send: GmailSend<GmailFiltersListResponse>,
}

impl GmailFiltersList {
    /// Builds the `users.settings.filters.list` request for the given user.
    pub fn new(auth: &HttpAuthBearer, user_id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail filters listing");
        trace!("user_id: {user_id:?}");

        let url = gmail_api_url(&["users", user_id, "settings", "filters"])?;
        let send = GmailSend::get(auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailFiltersList {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailFiltersListResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("filters listed");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
