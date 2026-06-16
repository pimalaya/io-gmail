use alloc::{format, string::String};

use log::{debug, trace};
use secrecy::SecretString;
use serde::Serialize;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::send::{GMAIL_API_BASE, GmailNoResponse, GmailSend, GmailSendError, GmailSendOutput},
};

#[derive(Debug, Serialize)]
struct GmailMessageBatchDeleteRequest<'a> {
    ids: &'a [String],
}

/// Gmail REST batch message deletion, yielding no response body.
pub struct GmailMessageBatchDelete {
    send: GmailSend<GmailNoResponse>,
}

impl GmailMessageBatchDelete {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        ids: &[String],
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail messages batch deletion");
        trace!("ids: {ids:?}");

        let url =
            Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/messages/batchDelete"))?;
        let body = GmailMessageBatchDeleteRequest { ids };
        let send = GmailSend::post_json(http_auth, url, &body)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailMessageBatchDelete {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailNoResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail messages batch deleted");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
