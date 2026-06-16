//! Patch a Gmail label (`users.labels.patch`).

use alloc::format;

use log::{debug, trace};
use secrecy::SecretString;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::labels::GmailLabel,
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

pub struct GmailLabelPatch {
    send: GmailSend<GmailLabel>,
}

impl GmailLabelPatch {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        label: &GmailLabel,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail label patch");
        trace!("label: {label:?}");

        let id = &label.id;
        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/labels/{id}"))?;
        let send = GmailSend::patch_json(http_auth, url, label)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailLabelPatch {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailLabel>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail label patched");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
