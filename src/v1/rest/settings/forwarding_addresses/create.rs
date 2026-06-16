use core::fmt;

use alloc::format;

use log::trace;
use secrecy::SecretString;
use url::Url;

use crate::{
    coroutine::*,
    gmail_try,
    v1::rest::settings::forwarding_addresses::GmailForwardingAddress,
    v1::send::{GMAIL_API_BASE, GmailSend, GmailSendError, GmailSendOutput},
};

pub struct GmailForwardingAddressCreate {
    state: State,
}

impl GmailForwardingAddressCreate {
    pub fn new(
        http_auth: &SecretString,
        user_id: &str,
        address: GmailForwardingAddress,
    ) -> Result<Self, GmailSendError> {
        let url = Url::parse(GMAIL_API_BASE)?
            .join(&format!("users/{user_id}/settings/forwardingAddresses"))?;

        Ok(Self {
            state: State::Send(GmailSend::post_json(http_auth, url, &address)?),
        })
    }
}

impl GmailCoroutine for GmailForwardingAddressCreate {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<GmailForwardingAddress>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        trace!("forwarding-address-create: {}", self.state);
        match &mut self.state {
            State::Send(send) => {
                let out = gmail_try!(send, arg);
                GmailCoroutineState::Complete(Ok(out))
            }
        }
    }
}

enum State {
    Send(GmailSend<GmailForwardingAddress>),
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Send(_) => f.write_str("send"),
        }
    }
}
