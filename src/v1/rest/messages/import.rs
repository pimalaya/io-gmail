use alloc::format;

use log::trace;
use secrecy::SecretString;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::rest::messages::{GmailInternalDateSource, GmailMessage},
    v1::send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
};

/// Gmail REST message import, wrapping the imported `GmailMessage`.
pub struct GmailMessageImport {
    send: GmailSend<GmailMessage>,
}

impl GmailMessageImport {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        message: &GmailMessage,
        internal_date_source: Option<GmailInternalDateSource>,
        deleted: bool,
        never_mark_spam: bool,
        process_for_calendar: bool,
    ) -> Result<Self, GmailSendError> {
        trace!("prepare gmail message import");

        let mut url =
            Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/messages/import"))?;

        {
            let mut query = url.query_pairs_mut();

            if let Some(internal_date_source) = internal_date_source {
                query.append_pair("internalDateSource", internal_date_source.as_str());
            }

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

        let send = GmailSend::post_json(http_auth, url, message)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailMessageImport {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailMessage>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        trace!("gmail message imported: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
