use core::fmt;

use alloc::{format, vec::Vec};

use log::trace;
use secrecy::SecretString;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::rest::threads::GmailThread,
    v1::send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
};

pub struct GmailThreadUntrash {
    state: State,
}

impl GmailThreadUntrash {
    pub fn new(http_auth: &SecretString, user_id: &str, id: &str) -> Result<Self, GmailSendError> {
        let url =
            Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/threads/{id}/untrash"))?;

        Ok(Self {
            state: State::Send(GmailSend::with_method(
                http_auth,
                "POST",
                url,
                None,
                Vec::new(),
            )),
        })
    }
}

impl GmailCoroutine for GmailThreadUntrash {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailThread>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        trace!("thread-untrash: {}", self.state);
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
