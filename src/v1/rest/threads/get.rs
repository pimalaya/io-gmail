use core::fmt;

use alloc::format;

use log::trace;
use secrecy::SecretString;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::rest::messages::GmailMessageFormat,
    v1::rest::threads::GmailThread,
    v1::send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
};

pub struct GmailThreadGet {
    state: State,
}

impl GmailThreadGet {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        id: &str,
        format: GmailMessageFormat,
        metadata_headers: &[&str],
    ) -> Result<Self, GmailSendError> {
        let mut url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/threads/{id}"))?;

        {
            let mut query = url.query_pairs_mut();
            query.append_pair("format", format.as_str());

            if matches!(format, GmailMessageFormat::Metadata) {
                for header in metadata_headers {
                    query.append_pair("metadataHeaders", header);
                }
            }
        }

        Ok(Self {
            state: State::Send(GmailSend::get(http_auth, url)),
        })
    }
}

impl GmailCoroutine for GmailThreadGet {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailThread>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        trace!("thread-get: {}", self.state);
        match &mut self.state {
            State::Send(send) => {
                let out = gmail_try!(send, arg);
                GmailCoroutineState::Complete(Ok(out))
            }
        }
    }
}

enum State {
    Send(GmailSend<GmailThread>),
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Send(_) => f.write_str("send"),
        }
    }
}
