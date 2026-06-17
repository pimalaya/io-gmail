//! Create a Gmail delegate (`users.settings.delegates.create`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.delegates/create>

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

pub struct GmailDelegateCreate {
    send: GmailSend<GmailDelegate>,
}

impl GmailDelegateCreate {
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        delegate: &GmailDelegate,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail delegate creation");
        trace!("delegate: {delegate:?}");

        let url =
            Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/settings/delegates"))?;
        let send = GmailSend::post_json(auth, url, delegate)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailDelegateCreate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailDelegate>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail delegate created");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
