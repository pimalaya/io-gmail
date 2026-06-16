use core::fmt;

use alloc::format;

use log::trace;
use secrecy::SecretString;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::send::{GMAIL_API_BASE, GmailNoResponse, GmailSend, GmailSendError, GmailSendOutput},
};

pub struct GmailDraftDelete {
    state: State,
}

impl GmailDraftDelete {
    pub fn new(http_auth: &SecretString, user_id: &str, id: &str) -> Result<Self, GmailSendError> {
        let url = Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/drafts/{id}"))?;

        Ok(Self {
            state: State::Send(GmailSend::delete(http_auth, url)),
        })
    }
}

impl GmailCoroutine for GmailDraftDelete {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailNoResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        trace!("draft-delete: {}", self.state);
        match &mut self.state {
            State::Send(send) => {
                let out = gmail_try!(send, arg);
                GmailCoroutineState::Complete(Ok(out))
            }
        }
    }
}

enum State {
    Send(GmailSend<GmailNoResponse>),
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Send(_) => f.write_str("send"),
        }
    }
}
