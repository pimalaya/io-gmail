//! Create a Gmail filter (`users.settings.filters.create`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.filters/create>

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        rest::settings::filters::GmailFilter,
        send::{GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
    },
};

/// I/O-free coroutine creating a filter on a Gmail account
/// (`users.settings.filters.create`).
pub struct GmailFilterCreate {
    send: GmailSend<GmailFilter>,
}

impl GmailFilterCreate {
    /// Builds the `users.settings.filters.create` request for the given
    /// filter.
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        filter: &GmailFilter,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail filter creation");
        trace!("filter: {filter:?}");

        let url = gmail_api_url(&["users", user_id, "settings", "filters"])?;
        let send = GmailSend::post_json(auth, url, filter)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailFilterCreate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailFilter>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("filter created");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
