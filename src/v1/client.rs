//! Std-blocking Gmail client: wraps a `Read + Write` stream plus the
//! bearer credential and drives the coroutines against
//! `gmail.googleapis.com`. Gated behind the `client` feature.

#[cfg(any(
    feature = "rustls-aws",
    feature = "rustls-ring",
    feature = "native-tls"
))]
use core::time::Duration;
use core::{any::Any, fmt};

#[cfg(any(
    feature = "rustls-aws",
    feature = "rustls-ring",
    feature = "native-tls"
))]
use alloc::string::ToString;
use alloc::{boxed::Box, string::String};

use std::io::{self, Read, Write};

#[cfg(any(
    feature = "rustls-aws",
    feature = "rustls-ring",
    feature = "native-tls"
))]
use pimalaya_stream::{std::stream::StreamStd, tls::Tls};
use secrecy::SecretString;
use thiserror::Error;
#[cfg(any(
    feature = "rustls-aws",
    feature = "rustls-ring",
    feature = "native-tls"
))]
use url::Url;

use crate::{
    coroutine::*,
    v1::rest::get_profile::{GmailProfile, GmailProfileGet},
    v1::rest::labels::{
        GmailLabel, GmailLabelsListResponse, create::GmailLabelCreate, delete::GmailLabelDelete,
        get::GmailLabelGet, list::GmailLabelsList, patch::GmailLabelPatch,
        update::GmailLabelUpdate,
    },
    v1::rest::messages::{
        GmailMessage, GmailMessageFormat, GmailMessageId, delete::GmailMessageDelete,
        get::GmailMessageGet, list::GmailMessagesList, list::GmailMessagesListResponse,
        modify::GmailMessageModify, send::GmailMessageSend, trash::GmailMessageTrash,
        untrash::GmailMessageUntrash,
    },
    v1::send::{GmailNoResponse, GmailSendError, GmailSendOutput},
};

#[derive(Debug, Error)]
pub enum GmailClientStdError {
    #[error(transparent)]
    Send(#[from] GmailSendError),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[cfg(any(
        feature = "rustls-aws",
        feature = "rustls-ring",
        feature = "native-tls"
    ))]
    #[error(transparent)]
    Tls(#[from] anyhow::Error),
    #[cfg(any(
        feature = "rustls-aws",
        feature = "rustls-ring",
        feature = "native-tls"
    ))]
    #[error("Gmail URL `{0}` has no host")]
    UrlMissingHost(String),
    #[cfg(any(
        feature = "rustls-aws",
        feature = "rustls-ring",
        feature = "native-tls"
    ))]
    #[error("Gmail URL `{0}` has unsupported scheme `{1}` (expected `http` or `https`)")]
    UrlUnsupportedScheme(String, String),
}

const READ_BUFFER_SIZE: usize = 16 * 1024;

pub struct GmailClientStd {
    pub stream: Box<dyn GmailStream>,
    pub http_auth: SecretString,
    pub user_id: String,
}

impl GmailClientStd {
    pub fn new<S: Read + Write + Send + 'static>(
        stream: S,
        http_auth: SecretString,
        user_id: impl Into<String>,
    ) -> Self {
        Self {
            stream: Box::new(stream),
            http_auth,
            user_id: user_id.into(),
        }
    }

    #[cfg(any(
        feature = "rustls-aws",
        feature = "rustls-ring",
        feature = "native-tls"
    ))]
    pub fn connect(
        url: &Url,
        tls: &Tls,
        http_auth: SecretString,
        user_id: impl Into<String>,
    ) -> Result<Self, GmailClientStdError> {
        let host = url
            .host_str()
            .ok_or_else(|| GmailClientStdError::UrlMissingHost(url.to_string()))?;

        let stream = match url.scheme() {
            "http" => StreamStd::connect_tcp(host, url.port().unwrap_or(80))?,
            "https" => StreamStd::connect_tls(host, url.port().unwrap_or(443), tls)?,
            scheme => {
                return Err(GmailClientStdError::UrlUnsupportedScheme(
                    url.to_string(),
                    scheme.to_string(),
                ));
            }
        };

        stream.set_read_timeout(Some(Duration::from_secs(30)))?;

        Ok(Self {
            stream: Box::new(stream),
            http_auth,
            user_id: user_id.into(),
        })
    }

    pub fn set_stream<S: Read + Write + Send + 'static>(&mut self, stream: S) {
        self.stream = Box::new(stream);
    }

    pub fn run<C, T>(&mut self, mut coroutine: C) -> Result<GmailSendOutput<T>, GmailClientStdError>
    where
        C: GmailCoroutine<Yield = GmailYield, Return = Result<GmailSendOutput<T>, GmailSendError>>,
    {
        let mut buf = [0u8; READ_BUFFER_SIZE];
        let mut arg: Option<&[u8]> = None;

        loop {
            match coroutine.resume(arg.take()) {
                GmailCoroutineState::Complete(Ok(out)) => return Ok(out),
                GmailCoroutineState::Complete(Err(err)) => return Err(err.into()),
                GmailCoroutineState::Yielded(GmailYield::WantsRead) => {
                    let n = self.stream.read(&mut buf)?;
                    arg = Some(&buf[..n]);
                }
                GmailCoroutineState::Yielded(GmailYield::WantsWrite(bytes)) => {
                    self.stream.write_all(&bytes)?;
                    arg = None;
                }
            }
        }
    }

    pub fn profile_get(&mut self) -> Result<GmailSendOutput<GmailProfile>, GmailClientStdError> {
        let coroutine = GmailProfileGet::new(&self.http_auth, &self.user_id)?;
        self.run(coroutine)
    }

    pub fn labels_list(
        &mut self,
    ) -> Result<GmailSendOutput<GmailLabelsListResponse>, GmailClientStdError> {
        let coroutine = GmailLabelsList::new(&self.http_auth, &self.user_id)?;
        self.run(coroutine)
    }

    pub fn label_get(
        &mut self,
        id: &str,
    ) -> Result<GmailSendOutput<GmailLabel>, GmailClientStdError> {
        let coroutine = GmailLabelGet::new(&self.http_auth, &self.user_id, id)?;
        self.run(coroutine)
    }

    pub fn label_create(
        &mut self,
        label: &GmailLabel,
    ) -> Result<GmailSendOutput<GmailLabel>, GmailClientStdError> {
        let coroutine = GmailLabelCreate::new(&self.http_auth, &self.user_id, label)?;
        self.run(coroutine)
    }

    pub fn label_update(
        &mut self,
        label: &GmailLabel,
    ) -> Result<GmailSendOutput<GmailLabel>, GmailClientStdError> {
        let coroutine = GmailLabelUpdate::new(&self.http_auth, &self.user_id, label)?;
        self.run(coroutine)
    }

    pub fn label_patch(
        &mut self,
        label: &GmailLabel,
    ) -> Result<GmailSendOutput<GmailLabel>, GmailClientStdError> {
        let coroutine = GmailLabelPatch::new(&self.http_auth, &self.user_id, label)?;
        self.run(coroutine)
    }

    pub fn label_delete(
        &mut self,
        id: &str,
    ) -> Result<GmailSendOutput<GmailNoResponse>, GmailClientStdError> {
        let coroutine = GmailLabelDelete::new(&self.http_auth, &self.user_id, id)?;
        self.run(coroutine)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn messages_list(
        &mut self,
        q: Option<&str>,
        label_ids: &[String],
        max_results: Option<u32>,
        page_token: Option<&str>,
        include_spam_trash: bool,
    ) -> Result<GmailSendOutput<GmailMessagesListResponse>, GmailClientStdError> {
        let coroutine = GmailMessagesList::new(
            &self.http_auth,
            &self.user_id,
            q,
            label_ids,
            max_results,
            page_token,
            include_spam_trash,
        )?;
        self.run(coroutine)
    }

    pub fn message_get(
        &mut self,
        id: &str,
        format: GmailMessageFormat,
        metadata_headers: &[&str],
    ) -> Result<GmailSendOutput<GmailMessage>, GmailClientStdError> {
        let coroutine =
            GmailMessageGet::new(&self.http_auth, &self.user_id, id, format, metadata_headers)?;
        self.run(coroutine)
    }

    pub fn message_send(
        &mut self,
        message: &GmailMessage,
    ) -> Result<GmailSendOutput<GmailMessageId>, GmailClientStdError> {
        let coroutine = GmailMessageSend::new(&self.http_auth, &self.user_id, message)?;
        self.run(coroutine)
    }

    pub fn message_modify(
        &mut self,
        id: &str,
        add_label_ids: &[String],
        remove_label_ids: &[String],
    ) -> Result<GmailSendOutput<GmailMessage>, GmailClientStdError> {
        let coroutine = GmailMessageModify::new(
            &self.http_auth,
            &self.user_id,
            id,
            add_label_ids,
            remove_label_ids,
        )?;
        self.run(coroutine)
    }

    pub fn message_trash(
        &mut self,
        id: &str,
    ) -> Result<GmailSendOutput<GmailMessage>, GmailClientStdError> {
        let coroutine = GmailMessageTrash::new(&self.http_auth, &self.user_id, id)?;
        self.run(coroutine)
    }

    pub fn message_untrash(
        &mut self,
        id: &str,
    ) -> Result<GmailSendOutput<GmailMessage>, GmailClientStdError> {
        let coroutine = GmailMessageUntrash::new(&self.http_auth, &self.user_id, id)?;
        self.run(coroutine)
    }

    pub fn message_delete(
        &mut self,
        id: &str,
    ) -> Result<GmailSendOutput<GmailNoResponse>, GmailClientStdError> {
        let coroutine = GmailMessageDelete::new(&self.http_auth, &self.user_id, id)?;
        self.run(coroutine)
    }
}

impl fmt::Debug for GmailClientStd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GmailClientStd")
            .field("http_auth", &self.http_auth)
            .field("user_id", &self.user_id)
            .finish_non_exhaustive()
    }
}

pub trait GmailStream: Read + Write + Send + Any {
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Read + Write + Send + Any> GmailStream for T {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
