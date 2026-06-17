//! Stop Gmail push notifications (`users.stop`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users/stop>

use alloc::{format, vec::Vec};

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::send::{GMAIL_API_BASE, GmailNoResponse, GmailSend, GmailSendError, GmailSendOutput},
};

pub struct GmailStop {
    send: GmailSend<GmailNoResponse>,
}

impl GmailStop {
    pub fn new(auth: &HttpAuthBearer, user_id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail watch stop");
        trace!("user_id: {user_id:?}");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/stop"))?;
        let send = GmailSend::with_method(auth, "POST", url, None, Vec::new());

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailStop {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailNoResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail watch stopped");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
