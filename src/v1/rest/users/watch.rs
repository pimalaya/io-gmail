//! Set up Gmail push notifications (`users.watch`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users/watch>

use alloc::format;

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::users::{GmailWatchRequest, GmailWatchResponse},
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

pub struct GmailWatch {
    send: GmailSend<GmailWatchResponse>,
}

impl GmailWatch {
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        request: &GmailWatchRequest,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail watch");
        trace!("request: {request:?}");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/watch"))?;
        let send = GmailSend::post_json(auth, url, request)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailWatch {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailWatchResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail watch established");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
