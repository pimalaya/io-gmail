//! Get a Gmail message (`users.messages.get`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.messages/get>

use alloc::format;

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::{
        query::to_field_pairs,
        rest::messages::{GmailMessage, GmailMessageFormat},
        send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    },
};

/// Gmail REST message retrieval, wrapping a `GmailMessage` response.
pub struct GmailMessageGet {
    send: GmailSend<GmailMessage>,
}

impl GmailMessageGet {
    /// Builds the `users.messages.get` request for the given message id.
    ///
    /// `metadata_headers` filters the returned headers, and only
    /// applies when the format is [`GmailMessageFormat::Metadata`].
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        id: &str,
        format: GmailMessageFormat,
        metadata_headers: &[&str],
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail message retrieval");
        trace!("id: {id:?}");

        let mut url =
            Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/messages/{id}"))?;

        {
            let mut query = url.query_pairs_mut();
            query.extend_pairs(to_field_pairs("format", &format));

            if matches!(format, GmailMessageFormat::Metadata) {
                for header in metadata_headers {
                    query.append_pair("metadataHeaders", header);
                }
            }
        }

        let send = GmailSend::get(auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailMessageGet {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailMessage>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        debug!("message retrieved");
        trace!("out: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
