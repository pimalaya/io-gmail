//! Get a Gmail filter (`users.settings.filters.get`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.filters/get>

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

/// I/O-free coroutine getting a filter of a Gmail account
/// (`users.settings.filters.get`).
pub struct GmailFilterGet {
    send: GmailSend<GmailFilter>,
}

impl GmailFilterGet {
    /// Builds the `users.settings.filters.get` request for the given filter
    /// id.
    pub fn new(auth: &HttpAuthBearer, user_id: &str, id: &str) -> Result<Self, GmailSendError> {
        debug!("prepare gmail filter retrieval");
        trace!("id: {id:?}");

        let url = gmail_api_url(&["users", user_id, "settings", "filters", id])?;
        let send = GmailSend::get(auth, url)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailFilterGet {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailFilter>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("filter retrieved");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
