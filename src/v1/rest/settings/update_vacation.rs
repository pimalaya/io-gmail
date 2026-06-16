use core::fmt;

use alloc::format;

use log::trace;
use secrecy::SecretString;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::rest::settings::GmailVacationSettings,
    v1::send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
};

pub struct GmailVacationUpdate {
    state: State,
}

impl GmailVacationUpdate {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        settings: GmailVacationSettings,
    ) -> Result<Self, GmailSendError> {
        let url =
            Url::parse(GMAIL_API_BASE)?.join(&format!("users/{user_id}/settings/vacation"))?;

        Ok(Self {
            state: State::Send(GmailSend::put_json(http_auth, url, &settings)?),
        })
    }
}

impl GmailCoroutine for GmailVacationUpdate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailVacationSettings>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        trace!("settings-vacation-update: {}", self.state);
        match &mut self.state {
            State::Send(send) => {
                let out = gmail_try!(send, arg);
                GmailCoroutineState::Complete(Ok(out))
            }
        }
    }
}

enum State {
    Send(GmailSend<GmailVacationSettings>),
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Send(_) => f.write_str("send"),
        }
    }
}
