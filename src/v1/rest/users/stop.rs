//! Stop Gmail push notifications (`users.stop`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users/stop>

use alloc::vec::Vec;

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};

use crate::{
    coroutine::*,
    gmail_try,
    v1::send::{GmailNoResponse, GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
};

/// I/O-free coroutine stopping Gmail push notifications (`users.stop`).
pub struct GmailStop {
    send: GmailSend<GmailNoResponse>,
}

impl GmailStop {
    /// Builds the `users.stop` request for the given user id (the
    /// mailbox owner, usually `me`).
    pub fn new(auth: &HttpAuthBearer, user_id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail watch stop");
        trace!("user_id: {user_id:?}");

        let url = gmail_api_url(&["users", user_id, "stop"])?;
        let send = GmailSend::with_method(auth, "POST", url, None, Vec::new())?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailStop {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailNoResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("watch stopped");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
