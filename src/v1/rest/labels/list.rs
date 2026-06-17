//! List the Gmail labels (`users.labels.list`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.labels/list>

use alloc::format;

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::labels::GmailLabelsListResponse,
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

pub struct GmailLabelsList {
    send: GmailSend<GmailLabelsListResponse>,
}

impl GmailLabelsList {
    pub fn new(auth: &HttpAuthBearer, user_id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail labels listing");
        trace!("user_id: {user_id:?}");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/labels"))?;
        let send = GmailSend::get(auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailLabelsList {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailLabelsListResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail labels listed");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
