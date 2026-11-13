//! Patch a Gmail label (`users.labels.patch`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.labels/patch>

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

/// I/O-free coroutine patching a Gmail label (`users.labels.patch`).
pub struct GmailLabelPatch {
    send: GmailSend<GmailLabel>,
}

impl GmailLabelPatch {
    /// Builds the `users.labels.patch` request from the given label,
    /// whose id selects the label to patch.
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        label: &GmailLabel,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail label patch");
        trace!("label: {label:?}");

        let id = &label.id;
        let url = gmail_api_url(&["users", user_id, "labels", id])?;
        let send = GmailSend::patch_json(auth, url, label)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailLabelPatch {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailLabel>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("label patched");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
