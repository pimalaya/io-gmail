//! Delete a Gmail label (`users.labels.delete`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.labels/delete>

use alloc::format;

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::send::{GMAIL_API_BASE, GmailNoResponse, GmailSend, GmailSendError, GmailSendOutput},
};

pub struct GmailLabelDelete {
    send: GmailSend<GmailNoResponse>,
}

impl GmailLabelDelete {
    pub fn new(auth: &HttpAuthBearer, user_id: &str, id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail label deletion");
        trace!("id: {id:?}");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/labels/{id}"))?;
        let send = GmailSend::delete(auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailLabelDelete {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailNoResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail label deleted");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
