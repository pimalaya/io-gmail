//! Create a Gmail label (`users.labels.create`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.labels/create>

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::labels::GmailLabel,
        send::{GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
    },
};

/// I/O-free coroutine creating a Gmail label (`users.labels.create`).
pub struct GmailLabelCreate {
    send: GmailSend<GmailLabel>,
}

impl GmailLabelCreate {
    /// Builds the `users.labels.create` request from the given label.
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

        let url = gmail_api_url(&["users", user_id, "labels"])?;
        let send = GmailSend::post_json(auth, url, label)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailLabelCreate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailLabel>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("label created");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
