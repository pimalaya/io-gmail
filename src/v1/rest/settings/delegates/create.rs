//! Create a Gmail delegate (`users.settings.delegates.create`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.delegates/create>

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::delegates::GmailDelegate,
        send::{GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
    },
};

/// I/O-free coroutine creating a delegate on a Gmail account
/// (`users.settings.delegates.create`).
pub struct GmailDelegateCreate {
    send: GmailSend<GmailDelegate>,
}

impl GmailDelegateCreate {
    /// Builds the `users.settings.delegates.create` request for the given
    /// delegate.
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        delegate: &GmailDelegate,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail delegate creation");
        trace!("delegate: {delegate:?}");

        let url = gmail_api_url(&["users", user_id, "settings", "delegates"])?;
        let send = GmailSend::post_json(auth, url, delegate)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailDelegateCreate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailDelegate>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("delegate created");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
