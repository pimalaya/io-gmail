use alloc::format;

use log::trace;
use secrecy::SecretString;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::rest::messages::{GmailMessage, GmailMessageFormat},
    v1::send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
};

/// Gmail REST message retrieval, wrapping a `GmailMessage` response.
pub struct GmailMessageGet {
    send: GmailSend<GmailMessage>,
}

impl GmailMessageGet {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        id: &str,
        format: GmailMessageFormat,
        metadata_headers: &[&str],
    ) -> Result<Self, GmailSendError> {
        trace!("prepare gmail message {id} retrieval");

        let mut url =
            Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/messages/{id}"))?;

        {
            let mut query = url.query_pairs_mut();
            query.append_pair("format", format.as_str());

            if matches!(format, GmailMessageFormat::Metadata) {
                for header in metadata_headers {
                    query.append_pair("metadataHeaders", header);
                }
            }
        }

        let send = GmailSend::get(http_auth, url);

        Ok(Self { send })
    }
}

impl GmailCoroutine for GmailMessageGet {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailMessage>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let out = gmail_try!(&mut self.send, arg);
        trace!("gmail message retrieved: {out:?}");
        GmailCoroutineState::Complete(Ok(out))
    }
}
