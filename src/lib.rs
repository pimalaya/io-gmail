#![no_std]
#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! # io-gmail
//!
//! I/O-free coroutines for the [Gmail REST API], built on [io-http]
//! (HTTP/1.1) and pumped by any stream the caller owns.
//!
//! [Gmail REST API]: https://developers.google.com/gmail/api/reference/rest
//! [io-http]: https://docs.rs/io-http
//!
//! io-gmail is the Gmail sibling of [io-imap] and [io-jmap]: same shape,
//! different wire protocol (JSON over HTTP rather than IMAP or JMAP). It
//! is consumed by io-email (as the Gmail backend of the shared email
//! API) and directly by himalaya (the protocol-specific gmail commands).
//!
//! [io-imap]: https://docs.rs/io-imap
//! [io-jmap]: https://docs.rs/io-jmap
//!
//! ## Layers and features
//!
//! The crate has two of the three standard Pimalaya layers; there is no
//! CLI:
//!
//! 1. **I/O-free coroutines** (`no_std` core, always present): the whole
//!    Gmail REST logic.
//! 2. **Std client** ([`v1::client::GmailClientStd`], `client` feature):
//!    a blocking pump over any stream, with `connect` opening the
//!    TCP/TLS connection itself behind a TLS feature (`rustls-ring` by
//!    default, `rustls-aws`, `native-tls`).
//!
//! ## Everything lives under v1
//!
//! The Gmail REST API is versioned (`/gmail/v1/`), so the crate is too:
//! the version-agnostic [`coroutine`] contract stays at the crate root,
//! everything else lives under [`v1`]. The day Gmail ships a v2, a
//! sibling module slots in without breaking `v1` consumers.
//!
//! ## The coroutine contract
//!
//! Every exchange implements [`coroutine::GmailCoroutine`]: `resume`
//! takes the bytes read since the last yield and either requests I/O
//! ([`coroutine::GmailYield`] `WantsRead` / `WantsWrite`) or completes.
//! The [`gmail_try!`] macro is the coroutine equivalent of `?`.
//!
//! A Gmail call is a single HTTP request/response, so every REST
//! coroutine is a thin wrapper around one shared primitive,
//! [`v1::send::GmailSend`]: it builds the authorized request (bearer
//! token, JSON in and out) and parses either the 2xx body or Gmail's
//! error envelope into [`v1::send::GmailSendError`]. The envelope is
//! kept whole in [`v1::send::GmailApiError`] (status, `errors[].reason`,
//! `status` and `details[].reason`), so callers match rate limits, quota
//! answers and gone resources on Google's codes rather than on the
//! message text. Redirects are never followed. The terminal
//! [`v1::send::GmailSendOutput`] carries the parsed response plus a
//! keep-alive flag so pumps can reuse the connection across the many
//! small requests a Gmail session makes.
//!
//! ## Naming
//!
//! Public items follow `<Domain><Target><Verb><Ext>`: the domain is
//! `Gmail`, the target-verb pair mirrors the REST method
//! (`GmailLabelGet` for `users.labels.get`, `GmailMessagesBatchDelete`
//! for `users.messages.batchDelete`) and the extension distinguishes
//! companions (`Params`, `Response`, `Error`, `Yield`). Pure data
//! resources omit the verb (`GmailLabel`, `GmailMessage`); the target
//! is omitted when the verb applies to the whole exchange
//! ([`v1::send::GmailSend`], `GmailWatch`, `GmailStop`).
//!
//! ## Module layout
//!
//! [`v1::rest`] mirrors the Gmail REST reference one-to-one. The whole
//! API hangs off the `users` resource, so that level is flattened away:
//! each sub-resource is a directory and each method a file named after
//! the API method in snake_case (`getProfile` becomes get_profile.rs).
//! A reader who knows the reference knows where to look.
//!
//! Request bodies take the whole resource by reference (`&GmailLabel`,
//! `&GmailMessage`), so a `Default` resource with a few fields set
//! serializes cleanly. Enum-valued wire strings are typed enums. List
//! methods take borrowed `*Params` structs flattened into query pairs
//! by [`v1::query::to_query_pairs`], a tiny `no_std` serde serializer
//! that emits the repeated-key sequences Gmail expects.
//!
//! ## Watching a mailbox
//!
//! [`v1::history_poll::GmailHistoryPoll`] is the one composite,
//! multi-step coroutine: an infinite watch that baselines the history
//! cursor via `users.getProfile`, polls `users.history.list` on a timer
//! and yields one Gmail-native diff per tick, re-baselining on an
//! expired cursor. It is the polling alternative to `users.watch` and
//! `users.stop` (Pub/Sub push), which exist as plain coroutines for API
//! completeness but are not wired into a watcher.
//!
//! ## Authentication
//!
//! io-gmail does no OAuth itself: the API only accepts OAuth 2.0 bearer
//! tokens, so the credential is exactly a bare access token, and minting
//! or refreshing it is the caller's responsibility.
//!
//! ## Logging
//!
//! Coroutines pair a `debug!` lifecycle line with one `trace!` per input
//! variable in `new()`, and a `debug!` plus `trace!("out: ...")` when
//! `resume` completes; the crate never logs above `debug!`.
//!
//! ## Example
//!
//! Running a coroutine against a caller-owned TLS stream:
//!
//! ```rust,no_run
//! use std::{
//!     io::{Read, Write},
//!     net::TcpStream,
//!     sync::Arc,
//! };
//!
//! use io_gmail::{coroutine::*, v1::rest::users::get_profile::GmailProfileGet};
//! use io_http::rfc6750::bearer::HttpAuthBearer;
//! use rustls::{ClientConfig, ClientConnection, StreamOwned};
//! use rustls_platform_verifier::ConfigVerifierExt;
//!
//! let config = ClientConfig::with_platform_verifier().unwrap();
//! let server_name = "gmail.googleapis.com".try_into().unwrap();
//! let conn = ClientConnection::new(Arc::new(config), server_name).unwrap();
//! let tcp = TcpStream::connect(("gmail.googleapis.com", 443)).unwrap();
//! let mut stream = StreamOwned::new(conn, tcp);
//!
//! let auth = HttpAuthBearer::new("token");
//! let mut coroutine = GmailProfileGet::new(&auth, "me").unwrap();
//!
//! let mut arg: Option<&[u8]> = None;
//! let mut buf = [0u8; 8192];
//! let mut read = Vec::new();
//!
//! let out = loop {
//!     match coroutine.resume(arg.take()) {
//!         GmailCoroutineState::Complete(Ok(out)) => break out,
//!         GmailCoroutineState::Complete(Err(err)) => panic!("{err}"),
//!         GmailCoroutineState::Yielded(GmailYield::WantsRead) => {
//!             let n = stream.read(&mut buf).unwrap();
//!             read.clear();
//!             read.extend_from_slice(&buf[..n]);
//!             arg = Some(&read);
//!         }
//!         GmailCoroutineState::Yielded(GmailYield::WantsWrite(bytes)) => {
//!             stream.write_all(&bytes).unwrap();
//!         }
//!     }
//! };
//!
//! println!("email address: {}", out.response.email_address);
//! ```

extern crate alloc;
#[cfg(feature = "client")]
extern crate std;

pub mod coroutine;
pub mod v1;
