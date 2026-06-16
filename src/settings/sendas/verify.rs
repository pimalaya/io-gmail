use core::fmt;

use alloc::format;

use log::trace;
use secrecy::SecretString;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    send::{GMAIL_API_BASE, GmailNoResponse, GmailSend, GmailSendError, GmailSendOutput},
};

pub struct GmailSendAsVerify {
    state: State,
}

impl GmailSendAsVerify {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        send_as_email: &str,
    ) -> Result<Self, GmailSendError> {
        let url = Url::parse(GMAIL_API_BASE)?.join(&format!(
            "users/{user_id}/settings/sendAs/{send_as_email}/verify"
        ))?;

        Ok(Self {
            state: State::Send(GmailSend::with_method(
                http_auth,
                "POST",
                url,
                None,
                alloc::vec::Vec::new(),
            )),
        })
    }
}

impl GmailCoroutine for GmailSendAsVerify {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailNoResponse>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        trace!("send-as-verify: {}", self.state);
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
