//! Create a Gmail filter (`users.settings.filters.create`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.filters/create>

use alloc::format;

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::filters::GmailFilter,
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

pub struct GmailFilterCreate {
    send: GmailSend<GmailFilter>,
}

impl GmailFilterCreate {
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        filter: &GmailFilter,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail filter creation");
        trace!("filter: {filter:?}");

        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/settings/filters"))?;
        let send = GmailSend::post_json(auth, url, filter)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailFilterCreate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailFilter>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail filter created");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
