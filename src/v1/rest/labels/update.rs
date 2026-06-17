//! Update a Gmail label (`users.labels.update`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.labels/update>

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

pub struct GmailLabelUpdate {
    send: GmailSend<GmailLabel>,
}

impl GmailLabelUpdate {
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        label: &GmailLabel,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail label update");
        trace!("label: {label:?}");

        if label.name.trim().is_empty() {
            let err = GmailSendError::InvalidRequest("Label name cannot be empty".into());
            return Err(err);
        }

        let id = &label.id;
        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/labels/{id}"))?;
        let send = GmailSend::put_json(auth, url, label)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailLabelUpdate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailLabel>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail label updated");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
