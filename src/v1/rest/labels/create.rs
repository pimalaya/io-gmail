//! Create a Gmail label (`users.labels.create`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.labels/create>

use alloc::format;

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::labels::GmailLabel,
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

pub struct GmailLabelCreate {
    send: GmailSend<GmailLabel>,
}

impl GmailLabelCreate {
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        label: &GmailLabel,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail label for creation");
        trace!("label: {label:?}");

        if label.name.trim().is_empty() {
            let err = GmailSendError::InvalidRequest("Label name cannot be empty".into());
            return Err(err);
        }

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/labels"))?;
        let send = GmailSend::post_json(auth, url, label)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailLabelCreate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailLabel>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail label created");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
