//! Import a Gmail message (`users.messages.import`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.messages/import>

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        query::to_field_pairs,
        rest::messages::{GmailInternalDateSource, GmailMessage},
        send::{GmailSend, GmailSendError, GmailSendOutput, gmail_api_url},
    },
};

/// Gmail REST message import, wrapping the imported `GmailMessage`.
pub struct GmailMessageImport {
    send: GmailSend<GmailMessage>,
}

impl GmailMessageImport {
    /// Builds the `users.messages.import` request for the given message.
    ///
    /// `deleted` marks the message as permanently deleted (not
    /// trashed), visible only in Google Vault.
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        message: &GmailMessage,
        internal_date_source: Option<GmailInternalDateSource>,
        deleted: bool,
        never_mark_spam: bool,
        process_for_calendar: bool,
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail message import");
        trace!("message: {message:?}");

        let mut url = gmail_api_url(&["users", user_id, "messages", "import"])?;

        {
            let mut query = url.query_pairs_mut();

            query.extend_pairs(to_field_pairs("internalDateSource", &internal_date_source));

            if deleted {
                query.append_pair("deleted", "true");
            }

            if never_mark_spam {
                query.append_pair("neverMarkSpam", "true");
            }

            if process_for_calendar {
                query.append_pair("processForCalendar", "true");
            }
        }

        let send = GmailSend::post_json(auth, url, message)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailMessageImport {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailMessage>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("message imported");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
