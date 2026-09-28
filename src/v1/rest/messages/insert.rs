//! Insert a Gmail message (`users.messages.insert`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.messages/insert>

use alloc::format;

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        query::to_field_pairs,
        rest::messages::{GmailInternalDateSource, GmailMessage},
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

/// Gmail REST message insert, wrapping the inserted `GmailMessage`.
pub struct GmailMessageInsert {
    send: GmailSend<GmailMessage>,
}

impl GmailMessageInsert {
    /// Builds the `users.messages.insert` request for the given message.
    ///
    /// `deleted` marks the message as permanently deleted (not
    /// trashed), visible only in Google Vault.
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        message: &GmailMessage,
        internal_date_source: Option<GmailInternalDateSource>,
        deleted: bool,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail message insertion");
        trace!("message: {message:?}");

        let mut url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/messages"))?;

        {
            let mut query = url.query_pairs_mut();

            query.extend_pairs(to_field_pairs("internalDateSource", &internal_date_source));

            if deleted {
                query.append_pair("deleted", "true");
            }
        }

        let send = GmailSend::post_json(auth, url, message)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailMessageInsert {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailMessage>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("message inserted");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
