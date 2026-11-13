//! List the Gmail delegates (`users.settings.delegates.list`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.delegates/list>

use alloc::vec::Vec;

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use serde::{Deserialize, Serialize};

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::delegates::GmailDelegate,
        send::{GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
    },
};

/// Response wrapping the delegates of a Gmail account.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GmailDelegatesListResponse {
    /// Delegates of the Gmail account.
    #[serde(default)]
    pub delegates: Vec<GmailDelegate>,
}

/// I/O-free coroutine listing the delegates of a Gmail account
/// (`users.settings.delegates.list`).
pub struct GmailDelegatesList {
    send: GmailSend<GmailDelegatesListResponse>,
}

impl GmailDelegatesList {
    /// Builds the `users.settings.delegates.list` request for the given user.
    pub fn new(auth: &HttpAuthBearer, user_id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail delegates listing");
        trace!("user_id: {user_id:?}");

        let url = gmail_api_url(&["users", user_id, "settings", "delegates"])?;
        let send = GmailSend::get(auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailDelegatesList {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailDelegatesListResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("delegates listed");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
