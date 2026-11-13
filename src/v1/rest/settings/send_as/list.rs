//! List the Gmail send-as aliases (`users.settings.sendAs.list`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.sendAs/list>

use alloc::vec::Vec;

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use serde::{Deserialize, Serialize};

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::send_as::GmailSendAs,
        send::{GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
    },
};

/// Response wrapping the send-as aliases of a Gmail account.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GmailSendAsListResponse {
    /// Send-as aliases of the Gmail account.
    #[serde(default)]
    pub send_as: Vec<GmailSendAs>,
}

/// I/O-free coroutine listing the send-as aliases of a Gmail account
/// (`users.settings.sendAs.list`).
pub struct GmailSendAsList {
    send: GmailSend<GmailSendAsListResponse>,
}

impl GmailSendAsList {
    /// Builds the `users.settings.sendAs.list` request for the given user.
    pub fn new(auth: &HttpAuthBearer, user_id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail send-as aliases listing");
        trace!("user_id: {user_id:?}");

        let url = gmail_api_url(&["users", user_id, "settings", "sendAs"])?;
        let send = GmailSend::get(auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailSendAsList {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailSendAsListResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("send-as aliases listed");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
