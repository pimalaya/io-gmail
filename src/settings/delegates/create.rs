use core::fmt;

use alloc::format;

use log::trace;
use secrecy::SecretString;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
    settings::delegates::GmailDelegate,
};

pub struct GmailDelegateCreate {
    state: State,
}

impl GmailDelegateCreate {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        delegate: GmailDelegate,
    ) -> Result<Self, GmailSendError> {
        let url =
            Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/settings/delegates"))?;

        Ok(Self {
            state: State::Send(GmailSend::post_json(http_auth, url, &delegate)?),
        })
    }
}

impl GmailCoroutine for GmailDelegateCreate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailDelegate>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        trace!("delegate-create: {}", self.state);
        match &mut self.state {
            State::Send(send) => {
                let out = gmail_try!(send, arg);
                GmailCoroutineState::Complete(Ok(out))
            }
        }
    }
}

enum State {
    Send(GmailSend<GmailDelegate>),
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Send(_) => f.write_str("send"),
        }
    }
}
