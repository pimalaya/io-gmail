//! Update a Gmail label (`users.labels.update`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.labels/update>

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

/// I/O-free coroutine updating a Gmail label (`users.labels.update`).
pub struct GmailLabelUpdate {
    send: GmailSend<GmailLabel>,
}

impl GmailLabelUpdate {
    /// Builds the `users.labels.update` request from the given label,
    /// whose id selects the label to update.
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
        let url = gmail_api_url(&["users", user_id, "labels", id])?;
        let send = GmailSend::put_json(auth, url, label)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailLabelUpdate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailLabel>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("label updated");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
