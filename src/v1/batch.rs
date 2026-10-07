//! Combine several Gmail calls into one HTTP request (`POST
//! /batch/gmail/v1`).
//!
//! The request body is a `multipart/mixed` document holding one
//! `application/http` part per inner call, each tagged with a
//! `Content-ID`; the answer is another `multipart/mixed` document whose
//! parts carry each inner HTTP response, tagged `response-<id>`, in no
//! guaranteed order. [`GmailBatch`] writes the former and parses the
//! latter back into one [`GmailBatchResponse`] per inner call, in the
//! order of the requests.
//!
//! A batch saves round trips, not quota: Google bills every inner call
//! its own units, and answers each one on its own. Some may succeed
//! while others are rate limited (429, or 403 `rateLimitExceeded`) or
//! gone (404), so the caller retries or drops them one by one
//! ([`GmailBatchResponseError::is_retryable`],
//! [`GmailBatchResponseError::is_not_found`]).
//!
//! Inner calls are `GET` requests on the Gmail API, such as
//! `users.messages.get` ([`GmailBatchRequest::message_get`]) or any URL
//! built under [`GMAIL_API_BASE`](crate::v1::send::GMAIL_API_BASE)
//! ([`GmailBatchRequest::get`]). The outer request carries the bearer
//! token; inner calls inherit it.
//!
//! <https://developers.google.com/gmail/api/guides/batch>
//!
//! ## Example
//!
//! Reading the metadata of a page of listed messages, then sorting the
//! answers into read, gone and to retry:
//!
//! ```rust,no_run
//! use std::io::{Read, Write};
//!
//! use io_gmail::{
//!     coroutine::*,
//!     v1::{
//!         batch::{GmailBatch, GmailBatchRequest},
//!         rest::messages::{GmailMessage, GmailMessageFormat},
//!     },
//! };
//! use io_http::rfc6750::bearer::HttpAuthBearer;
//!
//! # fn stream() -> std::net::TcpStream { unimplemented!() }
//! // A stream connected to www.googleapis.com:443, TLS included.
//! let mut stream = stream();
//!
//! let auth = HttpAuthBearer::new("token");
//! let ids = ["18c2f0a1b2c3d4e5", "18c2f0a1b2c3d4e6"];
//! let requests = ids
//!     .iter()
//!     .map(|id| {
//!         let headers = ["From", "Subject", "Date"];
//!         let format = GmailMessageFormat::Metadata;
//!         GmailBatchRequest::message_get(*id, "me", id, format, &headers)
//!     })
//!     .collect::<Result<Vec<_>, _>>()
//!     .unwrap();
//!
//! let mut coroutine = GmailBatch::<GmailMessage>::new(&auth, &requests).unwrap();
//! let mut arg: Option<&[u8]> = None;
//! let mut buf = [0u8; 8192];
//!
//! let out = loop {
//!     match coroutine.resume(arg.take()) {
//!         GmailCoroutineState::Complete(Ok(out)) => break out,
//!         GmailCoroutineState::Complete(Err(err)) => panic!("{err}"),
//!         GmailCoroutineState::Yielded(GmailYield::WantsRead) => {
//!             let n = stream.read(&mut buf).unwrap();
//!             arg = Some(&buf[..n]);
//!         }
//!         GmailCoroutineState::Yielded(GmailYield::WantsWrite(bytes)) => {
//!             stream.write_all(&bytes).unwrap();
//!         }
//!     }
//! };
//!
//! for response in out.response {
//!     match response.result {
//!         Ok(message) => println!("{}: {:?}", response.id, message.label_ids),
//!         Err(err) if err.is_not_found() => println!("{}: gone", response.id),
//!         Err(err) if err.is_retryable() => println!("{}: retry alone", response.id),
//!         Err(err) => println!("{}: {err}", response.id),
//!     }
//! }
//! ```

use core::marker::PhantomData;

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use io_http::{
    coroutine::*,
    rfc6750::bearer::HttpAuthBearer,
    rfc9110::{
        request::HttpRequest,
        send::{HttpSendOutput, HttpSendYield},
    },
    rfc9112::send::Http11Send,
};
use log::{debug, trace};
use serde::de::DeserializeOwned;
use thiserror::Error;
use url::{Position, Url};

use crate::{
    coroutine::*,
    v1::{
        rest::messages::{GmailMessageFormat, get::GmailMessageGet},
        send::{GmailApiError, GmailSendError, GmailSendOutput},
    },
};

/// URL of the Gmail batch endpoint, as documented by Google.
pub const GMAIL_BATCH_URL: &str = "https://www.googleapis.com/batch/gmail/v1";

/// Maximum number of inner calls Google accepts in one batch.
///
/// Google advises 50 or fewer for Gmail: larger batches are more likely
/// to have parts answered with a rate limit error.
pub const GMAIL_BATCH_MAX_REQUESTS: usize = 100;

/// Boundary separating the parts of the request body.
const BOUNDARY: &str = "io_gmail_batch_5f0c2b8e7d4a";

/// Prefix Google puts before the `Content-ID` of each answered part.
const RESPONSE_ID_PREFIX: &str = "response-";

/// One inner call of a batch: a `GET` on the Gmail API.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct GmailBatchRequest {
    /// The id correlating the call with its answer, unique within the
    /// batch; sent as the part's `Content-ID`.
    pub id: String,
    /// The absolute path and query of the call, such as
    /// `/gmail/v1/users/me/messages/abc?format=metadata`.
    pub path: String,
}

impl GmailBatchRequest {
    /// Builds a `GET` call on the given Gmail API URL, keeping its path
    /// and query.
    pub fn get(id: impl ToString, url: &Url) -> Self {
        Self {
            id: id.to_string(),
            path: url[Position::BeforePath..].to_string(),
        }
    }

    /// Builds a `users.messages.get` call, the batched form of
    /// [`GmailMessageGet`].
    ///
    /// `metadata_headers` filters the returned headers, and only
    /// applies when the format is [`GmailMessageFormat::Metadata`].
    pub fn message_get(
        id: impl ToString,
        user_id: &str,
        message_id: &str,
        format: GmailMessageFormat,
        metadata_headers: &[&str],
    ) -> Result<Self, GmailSendError> {
        let url = GmailMessageGet::url(user_id, message_id, format, metadata_headers)?;
        Ok(Self::get(id, &url))
    }
}

/// The answer to one inner call of a batch.
#[derive(Debug)]
pub struct GmailBatchResponse<T> {
    /// The id of the call this answers.
    pub id: String,
    /// The HTTP status of the inner response; `None` when the batch
    /// answer holds no readable part for the call.
    pub status: Option<u16>,
    /// The parsed 2xx body, or the error of the inner call.
    pub result: Result<T, GmailBatchResponseError>,
}

/// Error of one inner call of a batch.
#[derive(Debug, Error)]
pub enum GmailBatchResponseError {
    /// Gmail answered the call with a non-2xx status.
    #[error("{0}")]
    Api(GmailApiError),
    /// The 2xx body could not be parsed as JSON.
    #[error("Gmail batch response parsing failed: {0}")]
    ParseResponse(#[source] serde_json::Error),
    /// The batch answer holds no part for the call, or one without a
    /// readable HTTP status line.
    #[error("Gmail batch answer holds no response for this call")]
    Missing,
}

impl GmailBatchResponseError {
    /// Returns the API error when Gmail answered the call with a
    /// non-2xx status.
    pub fn api(&self) -> Option<&GmailApiError> {
        match self {
            Self::Api(err) => Some(err),
            _ => None,
        }
    }

    /// Returns the HTTP status code when the error is an API error.
    pub fn status(&self) -> Option<u16> {
        self.api().map(|err| err.status)
    }

    /// Whether Google throttled the call
    /// ([`GmailApiError::is_rate_limited`]).
    pub fn is_rate_limited(&self) -> bool {
        self.api().is_some_and(GmailApiError::is_rate_limited)
    }

    /// Whether the call is worth sending again on its own: rate
    /// limited, a transient 5xx ([`GmailApiError::is_retryable`]), or
    /// left unanswered by the batch.
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Missing) || self.api().is_some_and(GmailApiError::is_retryable)
    }

    /// Whether the target of the call does not exist (404), such as a
    /// message deleted since it was listed.
    pub fn is_not_found(&self) -> bool {
        self.status() == Some(404)
    }
}

/// I/O-free coroutine sending a Gmail batch and parsing each inner
/// answer into `T`.
///
/// Every inner call shares the response type `T`; batch calls of
/// different kinds with `serde_json::Value`.
pub struct GmailBatch<T> {
    ids: Vec<String>,
    send: Http11Send,
    _phantom: PhantomData<T>,
}

impl<T: DeserializeOwned> GmailBatch<T> {
    /// Sends `requests` to [`GMAIL_BATCH_URL`], between 1 and
    /// [`GMAIL_BATCH_MAX_REQUESTS`] with unique ids.
    pub fn new(
        auth: &HttpAuthBearer,
        requests: &[GmailBatchRequest],
    ) -> Result<Self, GmailSendError> {
        Self::with_url(auth, Url::parse(GMAIL_BATCH_URL)?, requests)
    }

    /// Same as [`Self::new`], against another batch endpoint URL.
    ///
    /// Google documents [`GMAIL_BATCH_URL`]; this lets a caller target
    /// the same path on another host, or a test server.
    pub fn with_url(
        auth: &HttpAuthBearer,
        url: Url,
        requests: &[GmailBatchRequest],
    ) -> Result<Self, GmailSendError> {
        debug!("prepare gmail batch");
        trace!("url: {url}");
        trace!("requests: {requests:?}");

        validate(requests)?;

        let mut body = Vec::new();

        for request in requests {
            body.extend_from_slice(format!("--{BOUNDARY}\r\n").as_bytes());
            body.extend_from_slice(b"Content-Type: application/http\r\n");
            body.extend_from_slice(format!("Content-ID: <{}>\r\n\r\n", request.id).as_bytes());
            body.extend_from_slice(format!("GET {} HTTP/1.1\r\n\r\n\r\n", request.path).as_bytes());
        }

        body.extend_from_slice(format!("--{BOUNDARY}--\r\n").as_bytes());

        let host = url.host_str().unwrap_or("localhost").to_string();
        let content_type = format!("multipart/mixed; boundary={BOUNDARY}");

        let mut request = HttpRequest::get(url)
            .header("Host", host)
            .header("Authorization", auth.to_authorization())
            .header("Content-Type", content_type)
            .body(body);

        request.method = "POST".into();

        let ids = requests.iter().map(|request| request.id.clone()).collect();

        Ok(Self {
            ids,
            send: Http11Send::new(request),
            _phantom: PhantomData,
        })
    }
}

impl<T: DeserializeOwned> GmailCoroutine for GmailBatch<T> {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<Vec<GmailBatchResponse<T>>>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        let HttpSendOutput {
            response,
            keep_alive,
            ..
        } = match self.send.resume(arg) {
            HttpCoroutineState::Yielded(HttpSendYield::WantsRead) => {
                return GmailCoroutineState::Yielded(GmailYield::WantsRead);
            }
            HttpCoroutineState::Yielded(HttpSendYield::WantsWrite(bytes)) => {
                return GmailCoroutineState::Yielded(GmailYield::WantsWrite(bytes));
            }
            HttpCoroutineState::Yielded(HttpSendYield::WantsRedirect { .. }) => {
                return GmailCoroutineState::Complete(Err(GmailSendError::UnexpectedRedirect));
            }
            HttpCoroutineState::Complete(Err(err)) => {
                return GmailCoroutineState::Complete(Err(err.into()));
            }
            HttpCoroutineState::Complete(Ok(out)) => out,
        };

        if !response.status.is_success() {
            let err = GmailApiError::parse(*response.status, &response.body);
            return GmailCoroutineState::Complete(Err(GmailSendError::Api(err)));
        }

        let Some(boundary) = response.header("content-type").and_then(multipart_boundary) else {
            let err = invalid_response("Gmail batch answer is not multipart/mixed");
            return GmailCoroutineState::Complete(Err(err));
        };

        let responses = parse_responses(&self.ids, &response.body, boundary);

        debug!("batch sent");
        trace!(
            "out: {:?}",
            responses
                .iter()
                .map(|response| (&response.id, response.status))
                .collect::<Vec<_>>()
        );

        GmailCoroutineState::Complete(Ok(GmailSendOutput {
            response: responses,
            keep_alive,
        }))
    }
}

/// Refuses an empty or oversized batch, duplicate ids, and ids or paths
/// that would break the multipart body.
fn validate(requests: &[GmailBatchRequest]) -> Result<(), GmailSendError> {
    if requests.is_empty() || requests.len() > GMAIL_BATCH_MAX_REQUESTS {
        return Err(GmailSendError::InvalidRequest(format!(
            "Batch must hold between 1 and {GMAIL_BATCH_MAX_REQUESTS} requests"
        )));
    }

    for (i, request) in requests.iter().enumerate() {
        let id = &request.id;

        if id.is_empty()
            || id.contains(BOUNDARY)
            || id
                .chars()
                .any(|c| c.is_whitespace() || c.is_control() || c == '<' || c == '>')
        {
            return Err(GmailSendError::InvalidRequest(format!(
                "Batch request id `{id}` is not a valid Content-ID"
            )));
        }

        if requests[..i].iter().any(|prev| prev.id == *id) {
            return Err(GmailSendError::InvalidRequest(format!(
                "Batch request id `{id}` is not unique"
            )));
        }

        let path = &request.path;

        if !path.starts_with('/')
            || path.contains(BOUNDARY)
            || path.chars().any(|c| c.is_whitespace() || c.is_control())
        {
            return Err(GmailSendError::InvalidRequest(format!(
                "Batch request path `{path}` is not a valid request target"
            )));
        }
    }

    Ok(())
}

/// Builds a response parsing error out of a plain message.
fn invalid_response(message: &str) -> GmailSendError {
    GmailSendError::ParseResponse(<serde_json::Error as serde::de::Error>::custom(message))
}

/// Reads the boundary of a `multipart/mixed` content type.
fn multipart_boundary(content_type: &str) -> Option<&str> {
    let mut params = content_type.split(';');

    let mime = params.next()?.trim();
    if !mime.eq_ignore_ascii_case("multipart/mixed") {
        return None;
    }

    params
        .filter_map(|param| param.split_once('='))
        .find(|(name, _)| name.trim().eq_ignore_ascii_case("boundary"))
        .map(|(_, value)| value.trim().trim_matches('"'))
        .filter(|boundary| !boundary.is_empty())
}

/// Matches the answered parts back to the requests by `Content-ID`,
/// answering in the order of `ids`.
fn parse_responses<T: DeserializeOwned>(
    ids: &[String],
    body: &[u8],
    boundary: &str,
) -> Vec<GmailBatchResponse<T>> {
    let mut answers: Vec<Option<(u16, &[u8])>> = ids.iter().map(|_| None).collect();

    for part in split_parts(body, boundary) {
        let (head, rest) = split_head(part);

        let Some(content_id) = header(head, "content-id") else {
            trace!("skip batch part without Content-ID");
            continue;
        };

        let content_id = content_id
            .trim()
            .trim_start_matches('<')
            .trim_end_matches('>');
        let content_id = content_id
            .strip_prefix(RESPONSE_ID_PREFIX)
            .unwrap_or(content_id);

        let Some(i) = ids.iter().position(|id| id == content_id) else {
            trace!("skip batch part for unknown Content-ID {content_id:?}");
            continue;
        };

        if answers[i].is_some() {
            trace!("skip duplicate batch part for Content-ID {content_id:?}");
            continue;
        }

        let Some(answer) = parse_inner_response(rest) else {
            trace!("skip batch part without HTTP status line for {content_id:?}");
            continue;
        };

        answers[i] = Some(answer);
    }

    ids.iter()
        .zip(answers)
        .map(|(id, answer)| {
            let Some((status, body)) = answer else {
                return GmailBatchResponse {
                    id: id.clone(),
                    status: None,
                    result: Err(GmailBatchResponseError::Missing),
                };
            };

            let result = if (200..300).contains(&status) {
                // NOTE: same normalisation as `GmailSend`: an empty 2xx
                // body reads as `{}`.
                let body = if body.iter().all(u8::is_ascii_whitespace) {
                    b"{}".as_slice()
                } else {
                    body
                };

                serde_json::from_slice(body).map_err(GmailBatchResponseError::ParseResponse)
            } else {
                Err(GmailBatchResponseError::Api(GmailApiError::parse(
                    status, body,
                )))
            };

            GmailBatchResponse {
                id: id.clone(),
                status: Some(status),
                result,
            }
        })
        .collect()
}

/// Reads the status code and body of an inner HTTP response.
fn parse_inner_response(bytes: &[u8]) -> Option<(u16, &[u8])> {
    let start = bytes.iter().position(|b| !b.is_ascii_whitespace())?;
    let (head, body) = split_head(&bytes[start..]);

    let status_line = lines(head).next()?;
    let mut tokens = status_line.split_ascii_whitespace();

    if !tokens.next()?.starts_with("HTTP/") {
        return None;
    }

    let status = tokens.next()?.parse().ok()?;
    Some((status, body))
}

/// Splits a multipart body into the contents of its parts.
///
/// Delimiters are only recognised at the start of a line, the line
/// break before each one belongs to it (RFC 2046 §5.1.1), and both CRLF
/// and bare LF line breaks are accepted. A body missing its close
/// delimiter ends its last part at the end of the body.
fn split_parts<'a>(body: &'a [u8], boundary: &str) -> Vec<&'a [u8]> {
    let delimiter = format!("--{boundary}");
    let delimiter = delimiter.as_bytes();

    let mut starts = Vec::new();
    let mut from = 0;

    while let Some(offset) = find(&body[from..], delimiter) {
        let pos = from + offset;

        if pos == 0 || body[pos - 1] == b'\n' {
            starts.push(pos);
        }

        from = pos + delimiter.len();
    }

    let mut parts = Vec::new();

    for (n, &pos) in starts.iter().enumerate() {
        let after = pos + delimiter.len();

        if body[after..].starts_with(b"--") {
            break;
        }

        let Some(offset) = body[after..].iter().position(|&b| b == b'\n') else {
            break;
        };

        let content_start = after + offset + 1;

        let content_end = match starts.get(n + 1) {
            None => body.len(),
            Some(&next) => {
                let mut end = next - 1;
                if end > 0 && body[end - 1] == b'\r' {
                    end -= 1;
                }
                end
            }
        };

        parts.push(&body[content_start..content_end.max(content_start)]);
    }

    parts
}

/// Splits a header block from what follows the first empty line.
fn split_head(bytes: &[u8]) -> (&[u8], &[u8]) {
    let mut pos = 0;

    while let Some(offset) = bytes[pos..].iter().position(|&b| b == b'\n') {
        let end = pos + offset;
        let line = &bytes[pos..end];

        if line.is_empty() || line == b"\r" {
            return (&bytes[..pos], &bytes[end + 1..]);
        }

        pos = end + 1;
    }

    (bytes, &[])
}

/// Iterates the lines of a header block, line breaks stripped.
fn lines(head: &[u8]) -> impl Iterator<Item = &str> {
    head.split(|&b| b == b'\n')
        .map(|line| line.strip_suffix(b"\r").unwrap_or(line))
        .filter_map(|line| core::str::from_utf8(line).ok())
}

/// Returns the value of the first header matching `name`
/// (case-insensitive).
fn header<'a>(head: &'a [u8], name: &str) -> Option<&'a str> {
    lines(head)
        .filter_map(|line| line.split_once(':'))
        .find(|(key, _)| key.trim().eq_ignore_ascii_case(name))
        .map(|(_, value)| value.trim())
}

/// Finds the first occurrence of `needle` in `haystack`.
fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
