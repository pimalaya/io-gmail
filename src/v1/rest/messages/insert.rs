//! Insert a Gmail message (`users.messages.insert`).

use alloc::format;

use log::{debug, trace};
use secrecy::SecretString;
use serde_variant::to_variant_name;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::rest::messages::{GmailInternalDateSource, GmailMessage},
    v1::send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
};

/// Gmail REST message insert, wrapping the inserted `GmailMessage`.
pub struct GmailMessageInsert {
    send: GmailSend<GmailMessage>,
}

impl GmailMessageInsert {
    pub fn new(
        http_auth: &SecretString,
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

            if let Some(ids) = internal_date_source {
                query.append_pair(
                    "internalDateSource",
                    to_variant_name(&ids).unwrap_or_default(),
                );
            }

            if deleted {
                query.append_pair("deleted", "true");
            }
        }

        let send = GmailSend::post_json(http_auth, url, message)?;

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailMessageInsert {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailMessage>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("gmail message inserted");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
