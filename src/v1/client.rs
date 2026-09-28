//! Std-blocking Gmail client, gated behind the `client` feature.
//!
//! Wraps a `Read + Write` stream plus the bearer credential and runs
//! the coroutines against `gmail.googleapis.com`.

#[cfg(any(
    feature = "rustls-aws",
    feature = "rustls-ring",
    feature = "native-tls"
))]
use core::time::Duration;
use core::{any::Any, fmt};

use alloc::{
    boxed::Box,
    string::{String, ToString},
};
use std::io::{self, Read, Write};

use io_http::rfc6750::bearer::HttpAuthBearer;
#[cfg(any(
    feature = "rustls-aws",
    feature = "rustls-ring",
    feature = "native-tls"
))]
use pimalaya_stream::{
    stream::{Stream, TcpConnectOptions, TlsConnectOptions},
    tls::Tls,
};
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
    v1::{
        rest::{
            labels::{
                GmailLabel,
                create::GmailLabelCreate,
                delete::GmailLabelDelete,
                get::GmailLabelGet,
                list::{GmailLabelsList, GmailLabelsListResponse},
                patch::GmailLabelPatch,
                update::GmailLabelUpdate,
            },
            messages::{
                GmailMessage, GmailMessageFormat, GmailMessageId,
                delete::GmailMessageDelete,
                get::GmailMessageGet,
                list::{GmailMessagesList, GmailMessagesListParams, GmailMessagesListResponse},
                modify::GmailMessageModify,
                send::GmailMessageSend,
                trash::GmailMessageTrash,
                untrash::GmailMessageUntrash,
            },
            users::{
                get_profile::{GmailProfile, GmailProfileGet},
                stop::GmailStop,
                watch::{GmailWatch, GmailWatchRequest, GmailWatchResponse},
            },
        },
        send::{GmailNoResponse, GmailSendError, GmailSendOutput},
    },
};

/// Errors that can occur on the std client.
#[derive(Debug, Error)]
pub enum GmailClientStdError {
    /// The Gmail exchange itself failed.
    #[error(transparent)]
    Send(#[from] GmailSendError),
    /// Reading from or writing to the stream failed.
    #[error(transparent)]
    Io(#[from] io::Error),
    /// Opening the TCP/TLS connection failed.
    #[cfg(any(
        feature = "rustls-aws",
        feature = "rustls-ring",
        feature = "native-tls"
    ))]
    #[error(transparent)]
    Tls(#[from] anyhow::Error),
    /// The API base URL carries no host to connect to.
    #[cfg(any(
        feature = "rustls-aws",
        feature = "rustls-ring",
        feature = "native-tls"
    ))]
    #[error("Gmail URL `{0}` has no host")]
    UrlMissingHost(String),
    /// The API base URL scheme is neither http nor https.
    #[cfg(any(
        feature = "rustls-aws",
        feature = "rustls-ring",
        feature = "native-tls"
    ))]
    #[error("Gmail URL `{url}` has unsupported scheme `{scheme}` (expected `http` or `https`)")]
    UrlUnsupportedScheme {
        /// The offending URL.
        url: String,
        /// The unsupported scheme it carries.
        scheme: String,
    },
}

/// Optional settings for [`GmailClientStd::connect`]; every field has a
/// default (the TLS backend default, and `me` as the mailbox owner).
pub struct GmailClientStdConnectOptions {
    /// TLS backend configuration.
    #[cfg(any(
        feature = "rustls-aws",
        feature = "rustls-ring",
        feature = "native-tls"
    ))]
    pub tls: Tls,
    /// Owner of the mailbox the requests target (`me` by default).
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

/// Standard, blocking Gmail client.
///
/// Owns the stream, the bearer credential and the mailbox owner; each
/// convenience method builds the matching coroutine and runs it to
/// completion. Coroutines without a convenience method go through
/// [`GmailClientStd::run`].
pub struct GmailClientStd {
    /// The underlying TCP or TLS stream.
    pub stream: Box<dyn GmailStream>,
    /// The OAuth 2.0 bearer credential added to every request.
    pub auth: HttpAuthBearer,
    /// Owner of the mailbox the requests target (usually `me`).
    pub user_id: String,
}

impl GmailClientStd {
    /// Builds a client over an already-connected stream.
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

    /// Opens a TCP/TLS connection to `gmail.googleapis.com` and builds
    /// the client around it.
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
            "http" => {
                let port = url.port().unwrap_or(80);
                let opts = TcpConnectOptions::default();
                Stream::connect_tcp(host, port, opts)?
            }
            "https" => {
                let port = url.port().unwrap_or(443);
                let opts = TlsConnectOptions {
                    tls: tls.clone(),
                    ..Default::default()
                };

                Stream::connect_tls(host, port, opts)?
            }
            scheme => {
                return Err(GmailClientStdError::UrlUnsupportedScheme {
                    url: url.to_string(),
                    scheme: scheme.to_string(),
                });
            }
        };

        stream.set_read_timeout(Some(Duration::from_secs(30)))?;

        Ok(Self {
            stream: Box::new(stream),
            auth: HttpAuthBearer::new(token.to_string()),
            user_id,
        })
    }

    /// Replaces the underlying stream, e.g. after reconnecting.
    pub fn set_stream<S: Read + Write + Send + 'static>(&mut self, stream: S) {
        self.stream = Box::new(stream);
    }

    /// Runs the given coroutine to completion against the stream,
    /// reading on `WantsRead` and writing on `WantsWrite`.
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

    /// Gets the profile of the mailbox (`users.getProfile`).
    pub fn profile_get(&mut self) -> Result<GmailSendOutput<GmailProfile>, GmailClientStdError> {
        let coroutine = GmailProfileGet::new(&self.auth, &self.user_id)?;
        self.run(coroutine)
    }

    /// Sets up Pub/Sub push notifications (`users.watch`).
    pub fn watch(
        &mut self,
        request: &GmailWatchRequest,
    ) -> Result<GmailSendOutput<GmailWatchResponse>, GmailClientStdError> {
        let coroutine = GmailWatch::new(&self.auth, &self.user_id, request)?;
        self.run(coroutine)
    }

    /// Stops Pub/Sub push notifications (`users.stop`).
    pub fn stop(&mut self) -> Result<GmailSendOutput<GmailNoResponse>, GmailClientStdError> {
        let coroutine = GmailStop::new(&self.auth, &self.user_id)?;
        self.run(coroutine)
    }

    /// Lists the labels of the mailbox (`users.labels.list`).
    pub fn labels_list(
        &mut self,
    ) -> Result<GmailSendOutput<GmailLabelsListResponse>, GmailClientStdError> {
        let coroutine = GmailLabelsList::new(&self.auth, &self.user_id)?;
        self.run(coroutine)
    }

    /// Gets a label by id (`users.labels.get`).
    pub fn label_get(
        &mut self,
        id: &str,
    ) -> Result<GmailSendOutput<GmailLabel>, GmailClientStdError> {
        let coroutine = GmailLabelGet::new(&self.auth, &self.user_id, id)?;
        self.run(coroutine)
    }

    /// Creates the given label (`users.labels.create`).
    pub fn label_create(
        &mut self,
        label: &GmailLabel,
    ) -> Result<GmailSendOutput<GmailLabel>, GmailClientStdError> {
        let coroutine = GmailLabelCreate::new(&self.auth, &self.user_id, label)?;
        self.run(coroutine)
    }

    /// Updates the given label in place (`users.labels.update`).
    pub fn label_update(
        &mut self,
        label: &GmailLabel,
    ) -> Result<GmailSendOutput<GmailLabel>, GmailClientStdError> {
        let coroutine = GmailLabelUpdate::new(&self.auth, &self.user_id, label)?;
        self.run(coroutine)
    }

    /// Patches the given label (`users.labels.patch`).
    pub fn label_patch(
        &mut self,
        label: &GmailLabel,
    ) -> Result<GmailSendOutput<GmailLabel>, GmailClientStdError> {
        let coroutine = GmailLabelPatch::new(&self.auth, &self.user_id, label)?;
        self.run(coroutine)
    }

    /// Deletes a label by id (`users.labels.delete`).
    pub fn label_delete(
        &mut self,
        id: &str,
    ) -> Result<GmailSendOutput<GmailNoResponse>, GmailClientStdError> {
        let coroutine = GmailLabelDelete::new(&self.auth, &self.user_id, id)?;
        self.run(coroutine)
    }

    /// Lists message ids matching the params (`users.messages.list`).
    pub fn messages_list(
        &mut self,
        params: &GmailMessagesListParams,
    ) -> Result<GmailSendOutput<GmailMessagesListResponse>, GmailClientStdError> {
        let coroutine = GmailMessagesList::new(&self.auth, &self.user_id, params)?;
        self.run(coroutine)
    }

    /// Gets a message by id (`users.messages.get`).
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

    /// Sends the given message (`users.messages.send`).
    pub fn message_send(
        &mut self,
        message: &GmailMessage,
    ) -> Result<GmailSendOutput<GmailMessageId>, GmailClientStdError> {
        let coroutine = GmailMessageSend::new(&self.auth, &self.user_id, message)?;
        self.run(coroutine)
    }

    /// Adds and removes labels on a message (`users.messages.modify`).
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

    /// Moves a message to the trash (`users.messages.trash`).
    pub fn message_trash(
        &mut self,
        id: &str,
    ) -> Result<GmailSendOutput<GmailMessage>, GmailClientStdError> {
        let coroutine = GmailMessageTrash::new(&self.auth, &self.user_id, id)?;
        self.run(coroutine)
    }

    /// Restores a message from the trash (`users.messages.untrash`).
    pub fn message_untrash(
        &mut self,
        id: &str,
    ) -> Result<GmailSendOutput<GmailMessage>, GmailClientStdError> {
        let coroutine = GmailMessageUntrash::new(&self.auth, &self.user_id, id)?;
        self.run(coroutine)
    }

    /// Permanently deletes a message (`users.messages.delete`).
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

/// Boxable client stream: `Read + Write + Send` plus `Any` so callers
/// can downcast back to the concrete stream type.
pub trait GmailStream: Read + Write + Send + Any {
    /// Returns the stream as a mutable `Any` for downcasting.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Read + Write + Send + Any> GmailStream for T {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
