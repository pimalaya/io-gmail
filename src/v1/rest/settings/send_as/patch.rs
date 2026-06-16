use core::fmt;

use alloc::format;

use log::trace;
use secrecy::SecretString;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::rest::settings::send_as::GmailSendAs,
    v1::send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
};

pub struct GmailSendAsPatch {
    state: State,
}

impl GmailSendAsPatch {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        send_as_email: &str,
        send_as: GmailSendAs,
    ) -> Result<Self, GmailSendError> {
        let url = Url::parse(GMAIL_API_BASE)?
            .join(&format!("users/{user_id}/settings/sendAs/{send_as_email}"))?;

        Ok(Self {
            state: State::Send(GmailSend::patch_json(http_auth, url, &send_as)?),
        })
    }
}

impl GmailCoroutine for GmailSendAsPatch {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailSendAs>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        trace!("send-as-patch: {}", self.state);
        match &mut self.state {
            State::Send(send) => {
                let out = gmail_try!(send, arg);
                GmailCoroutineState::Complete(Ok(out))
            }
        }
    }
}

enum State {
    Send(GmailSend<GmailSendAs>),
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Send(_) => f.write_str("send"),
        }
    }
}
