//! Std-blocking Gmail client: wraps a `Read + Write` stream plus the
//! bearer credential and runs the coroutines against
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
use io_http::rfc6750::bearer::HttpAuthBearer;

use std::io::{self, Read, Write};

#[cfg(any(
    feature = "rustls-aws",
    feature = "rustls-ring",
    feature = "native-tls"
))]
use pimalaya_stream::std::stream::StreamStd;
#[cfg(any(
    feature = "rustls-aws",
    feature = "rustls-ring",
    feature = "native-tls"
))]
pub use pimalaya_stream::tls::*;
use thiserror::Error;
#[cfg(any(
    feature = "rustls-aws",
    feature = "rustls-ring",
    feature = "native-tls"
))]
use url::Url;

#[cfg(any(
    feature = "rustls-aws",
    feature = "rustls-ring",
    feature = "native-tls"
))]
use crate::v1::send::GMAIL_API_BASE;
use crate::{
    coroutine::*,
    v1::rest::labels::{
        GmailLabel, GmailLabelsListResponse, create::GmailLabelCreate, delete::GmailLabelDelete,
        get::GmailLabelGet, list::GmailLabelsList, patch::GmailLabelPatch,
        update::GmailLabelUpdate,
    },
    v1::rest::messages::{
        GmailMessage, GmailMessageFormat, GmailMessageId, delete::GmailMessageDelete,
        get::GmailMessageGet, list::GmailMessagesList, list::GmailMessagesListParams,
        list::GmailMessagesListResponse, modify::GmailMessageModify, send::GmailMessageSend,
        trash::GmailMessageTrash, untrash::GmailMessageUntrash,
    },
    v1::rest::users::{
        GmailProfile, GmailWatchRequest, GmailWatchResponse, get_profile::GmailProfileGet,
        stop::GmailStop, watch::GmailWatch,
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

/// Optional settings for [`GmailClientStd::connect`]; every field has a
/// default (the TLS backend default, and `me` as the mailbox owner).
pub struct GmailClientStdConnectOptions {
    #[cfg(any(
        feature = "rustls-aws",
        feature = "rustls-ring",
        feature = "native-tls"
    ))]
    pub tls: Tls,
    pub user_id: String,
}

impl Default for GmailClientStdConnectOptions {
    fn default() -> Self {
        Self {
            #[cfg(any(
                feature = "rustls-aws",
                feature = "rustls-ring",
                feature = "native-tls"
            ))]
            tls: Tls::default(),
            user_id: String::from("me"),
        }
    }
}

const READ_BUFFER_SIZE: usize = 16 * 1024;

pub struct GmailClientStd {
    pub stream: Box<dyn GmailStream>,
    pub auth: HttpAuthBearer,
    pub user_id: String,
}

impl GmailClientStd {
    pub fn new<S: Read + Write + Send + 'static>(
        stream: S,
        token: impl ToString,
        options: GmailClientStdConnectOptions,
    ) -> Self {
        Self {
            stream: Box::new(stream),
            auth: HttpAuthBearer::new(token.to_string()),
            user_id: options.user_id,
        }
    }

    #[cfg(any(
        feature = "rustls-aws",
        feature = "rustls-ring",
        feature = "native-tls"
    ))]
    pub fn connect(
        token: impl ToString,
        options: GmailClientStdConnectOptions,
    ) -> Result<Self, GmailClientStdError> {
        let GmailClientStdConnectOptions { tls, user_id } = options;

        let url = Url::parse(GMAIL_API_BASE).expect("Gmail API base URL is valid");
        let host = url
            .host_str()
            .ok_or_else(|| GmailClientStdError::UrlMissingHost(url.to_string()))?;

        let stream = match url.scheme() {
            "http" => StreamStd::connect_tcp(host, url.port().unwrap_or(80))?,
            "https" => StreamStd::connect_tls(host, url.port().unwrap_or(443), &tls)?,
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
            auth: HttpAuthBearer::new(token.to_string()),
            user_id,
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
        let coroutine = GmailProfileGet::new(&self.auth, &self.user_id)?;
        self.run(coroutine)
    }

    pub fn watch(
        &mut self,
        request: &GmailWatchRequest,
    ) -> Result<GmailSendOutput<GmailWatchResponse>, GmailClientStdError> {
        let coroutine = GmailWatch::new(&self.auth, &self.user_id, request)?;
        self.run(coroutine)
    }

    pub fn stop(&mut self) -> Result<GmailSendOutput<GmailNoResponse>, GmailClientStdError> {
        let coroutine = GmailStop::new(&self.auth, &self.user_id)?;
        self.run(coroutine)
    }

    pub fn labels_list(
        &mut self,
    ) -> Result<GmailSendOutput<GmailLabelsListResponse>, GmailClientStdError> {
        let coroutine = GmailLabelsList::new(&self.auth, &self.user_id)?;
        self.run(coroutine)
    }

    pub fn label_get(
        &mut self,
        id: &str,
    ) -> Result<GmailSendOutput<GmailLabel>, GmailClientStdError> {
        let coroutine = GmailLabelGet::new(&self.auth, &self.user_id, id)?;
        self.run(coroutine)
    }

    pub fn label_create(
        &mut self,
        label: &GmailLabel,
    ) -> Result<GmailSendOutput<GmailLabel>, GmailClientStdError> {
        let coroutine = GmailLabelCreate::new(&self.auth, &self.user_id, label)?;
        self.run(coroutine)
    }

    pub fn label_update(
        &mut self,
        label: &GmailLabel,
    ) -> Result<GmailSendOutput<GmailLabel>, GmailClientStdError> {
        let coroutine = GmailLabelUpdate::new(&self.auth, &self.user_id, label)?;
        self.run(coroutine)
    }

    pub fn label_patch(
        &mut self,
        label: &GmailLabel,
    ) -> Result<GmailSendOutput<GmailLabel>, GmailClientStdError> {
        let coroutine = GmailLabelPatch::new(&self.auth, &self.user_id, label)?;
        self.run(coroutine)
    }

    pub fn label_delete(
        &mut self,
        id: &str,
    ) -> Result<GmailSendOutput<GmailNoResponse>, GmailClientStdError> {
        let coroutine = GmailLabelDelete::new(&self.auth, &self.user_id, id)?;
        self.run(coroutine)
    }

    pub fn messages_list(
        &mut self,
        params: &GmailMessagesListParams,
    ) -> Result<GmailSendOutput<GmailMessagesListResponse>, GmailClientStdError> {
        let coroutine = GmailMessagesList::new(&self.auth, &self.user_id, params)?;
        self.run(coroutine)
    }

    pub fn message_get(
        &mut self,
        id: &str,
        format: GmailMessageFormat,
        metadata_headers: &[&str],
    ) -> Result<GmailSendOutput<GmailMessage>, GmailClientStdError> {
        let coroutine =
            GmailMessageGet::new(&self.auth, &self.user_id, id, format, metadata_headers)?;
        self.run(coroutine)
    }

    pub fn message_send(
        &mut self,
        message: &GmailMessage,
    ) -> Result<GmailSendOutput<GmailMessageId>, GmailClientStdError> {
        let coroutine = GmailMessageSend::new(&self.auth, &self.user_id, message)?;
        self.run(coroutine)
    }

    pub fn message_modify(
        &mut self,
        id: &str,
        add_label_ids: &[String],
        remove_label_ids: &[String],
    ) -> Result<GmailSendOutput<GmailMessage>, GmailClientStdError> {
        let coroutine = GmailMessageModify::new(
            &self.auth,
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
        let coroutine = GmailMessageTrash::new(&self.auth, &self.user_id, id)?;
        self.run(coroutine)
    }

    pub fn message_untrash(
        &mut self,
        id: &str,
    ) -> Result<GmailSendOutput<GmailMessage>, GmailClientStdError> {
        let coroutine = GmailMessageUntrash::new(&self.auth, &self.user_id, id)?;
        self.run(coroutine)
    }

    pub fn message_delete(
        &mut self,
        id: &str,
    ) -> Result<GmailSendOutput<GmailNoResponse>, GmailClientStdError> {
        let coroutine = GmailMessageDelete::new(&self.auth, &self.user_id, id)?;
        self.run(coroutine)
    }
}

impl fmt::Debug for GmailClientStd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GmailClientStd")
            .field("auth", &self.auth)
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
