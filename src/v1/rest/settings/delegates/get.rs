//! Get a Gmail delegate (`users.settings.delegates.get`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.delegates/get>

use alloc::format;

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::delegates::GmailDelegate,
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

pub struct GmailDelegateGet {
    send: GmailSend<GmailDelegate>,
}

impl GmailDelegateGet {
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        delegate_email: &str,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail delegate retrieval");
        trace!("delegate_email: {delegate_email:?}");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!(
            "users/{user_id}/settings/delegates/{delegate_email}"
        ))?;
        let send = GmailSend::get(auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailDelegateGet {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailDelegate>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail delegate retrieved");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
