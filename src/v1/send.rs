//! HTTP/JSON transport every Gmail coroutine delegates to.
//!
//! Builds the authorized request and parses the JSON response, or the
//! Gmail error envelope on failure.
//!
//! Gmail API reference: <https://developers.google.com/gmail/api/reference/rest>.

use core::{marker::PhantomData, time::Duration};

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use io_http::{
    coroutine::*,
    rfc3986::path::{HttpUrlBuildError, http_url_build},
    rfc6750::bearer::HttpAuthBearer,
    rfc9110::{
        request::HttpRequest,
        send::{HttpSendOutput, HttpSendYield},
    },
    rfc9112::send::{Http11Send, Http11SendError},
};
use log::{debug, trace};
use serde::{Deserialize, Deserializer, Serialize, de::DeserializeOwned};
use serde_json::Value;
use thiserror::Error;
use url::Url;

use crate::{coroutine::*, v1::query::GmailQueryError};

/// Base URL of the Gmail REST API v1.
pub const GMAIL_API_BASE: &str = "https://gmail.googleapis.com/gmail/v1/";

/// The Gmail API URL of `segments` under [`GMAIL_API_BASE`], each one
/// percent-encoded as a single path segment, so an id holding `/`, `?` or
/// `#` cannot reach another resource.
///
/// A segment the URL would drop or resolve names no resource and is
/// refused: an empty one, `.`, `..`, or one holding a tab or a line
/// break.
pub fn gmail_api_url(segments: &[&str]) -> Result<Url, GmailSendError> {
    http_url_build(GMAIL_API_BASE, segments).map_err(|err| match err {
        HttpUrlBuildError::InvalidBaseUrl(err, _) => GmailSendError::ParseUrl(err),
        HttpUrlBuildError::InvalidSegment(_) => GmailSendError::InvalidRequest(err.to_string()),
    })
}

/// Unit marker deserialized from empty 2xx bodies (DELETE, batch
/// operations, stop).
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct GmailNoResponse;

impl<'de> Deserialize<'de> for GmailNoResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let _ = serde::de::IgnoredAny::deserialize(deserializer)?;
        Ok(Self)
    }
}

/// Errors that can occur during a Gmail exchange.
#[derive(Debug, Error)]
pub enum GmailSendError {
    /// The underlying HTTP exchange failed.
    #[error("Gmail HTTP request failed: {0}")]
    Send(#[from] Http11SendError),
    /// The request body could not be serialized to JSON.
    #[error("Gmail request serialization failed: {0}")]
    SerializeRequest(#[source] serde_json::Error),
    /// The 2xx response body could not be parsed as JSON.
    #[error("Gmail response parsing failed: {0}")]
    ParseResponse(#[source] serde_json::Error),
    /// The request URL could not be built.
    #[error("Gmail URL parsing failed: {0}")]
    ParseUrl(#[from] url::ParseError),
    /// The request was rejected before being sent.
    #[error("Invalid Gmail request: {0}")]
    InvalidRequest(String),
    /// Gmail returned a non-2xx status with its error envelope.
    #[error("{0}")]
    Api(GmailApiError),
    /// The server answered with a redirect, which is never followed.
    #[error("Gmail server returned an unexpected redirect")]
    UnexpectedRedirect,
}

impl GmailSendError {
    /// Returns the API error when Gmail answered with a non-2xx status.
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

    /// Returns how long the server asked to wait before a retry
    /// ([`GmailApiError::retry_after`]).
    pub fn retry_after(&self) -> Option<Duration> {
        self.api().and_then(|err| err.retry_after)
    }

    /// Whether the error is an API error worth retrying later
    /// ([`GmailApiError::is_retryable`]).
    pub fn is_retryable(&self) -> bool {
        self.api().is_some_and(GmailApiError::is_retryable)
    }

    /// Whether the error is a rate limit answer
    /// ([`GmailApiError::is_rate_limited`]).
    pub fn is_rate_limited(&self) -> bool {
        self.api().is_some_and(GmailApiError::is_rate_limited)
    }

    /// Whether the target of the request does not exist (404).
    pub fn is_not_found(&self) -> bool {
        self.status() == Some(404)
    }

    /// Whether a `users.history.list` answer says the `startHistoryId`
    /// is too old or unknown (404): the history cursor expired, and
    /// the caller must start over from a full listing.
    ///
    /// Only meaningful on a history listing, where it is the same test
    /// as [`Self::is_not_found`].
    pub fn is_history_expired(&self) -> bool {
        self.is_not_found()
    }
}

impl From<GmailQueryError> for GmailSendError {
    fn from(err: GmailQueryError) -> Self {
        Self::InvalidRequest(err.to_string())
    }
}

/// Gmail's error envelope, read from a non-2xx answer.
///
/// Google answers errors as
/// `{"error":{"code":403,"message":"...","errors":[{"reason":"rateLimitExceeded",...}],"status":"PERMISSION_DENIED","details":[{"reason":"RATE_LIMIT_EXCEEDED",...}]}}`;
/// the reasons and statuses are kept so callers match on codes rather
/// than on the message text.
#[derive(Debug, Clone, Default, Eq, PartialEq, Error)]
#[error("Gmail API returned HTTP {status}: {message}")]
pub struct GmailApiError {
    /// The effective status code: `error.code` when present, the HTTP
    /// status otherwise.
    pub status: u16,
    /// The error message, from the envelope or a summary of the raw
    /// body.
    pub message: String,
    /// The reasons of `error.errors[]`, in order, such as `notFound`,
    /// `rateLimitExceeded`, `userRateLimitExceeded`, `quotaExceeded` or
    /// `dailyLimitExceeded`.
    pub reasons: Vec<String>,
    /// The canonical status of `error.status`, such as `NOT_FOUND`,
    /// `PERMISSION_DENIED` or `RESOURCE_EXHAUSTED`.
    pub google_status: Option<String>,
    /// The reasons of `error.details[]` (`google.rpc.ErrorInfo`), such
    /// as `RATE_LIMIT_EXCEEDED`.
    pub detail_reasons: Vec<String>,
    /// How long the server asked to wait before a retry: the
    /// `Retry-After` header in its seconds form, else the delay of
    /// `error.details[]` (`google.rpc.RetryInfo`).
    pub retry_after: Option<Duration>,
}

impl GmailApiError {
    /// Reads Gmail's error envelope out of a non-2xx body, falling back
    /// to a summary of the raw body as message.
    ///
    /// The body does not tell the `Retry-After` header: the exchange
    /// fills it in.
    pub fn parse(http_status: u16, body: &[u8]) -> Self {
        let (status, message) = parse_api_error(http_status, body);
        let mut err = Self {
            status,
            message,
            ..Default::default()
        };

        if let Ok(envelope) = serde_json::from_slice::<ErrorEnvelope>(body) {
            let error = envelope.error;
            err.reasons = error.errors.into_iter().filter_map(|e| e.reason).collect();
            err.google_status = error.status;
            err.retry_after = error
                .details
                .iter()
                .find_map(|d| d.retry_delay.as_ref()?.as_str().and_then(parse_delay));
            err.detail_reasons = error.details.into_iter().filter_map(|d| d.reason).collect();
        }

        err
    }

    /// Whether `reason` is one of [`Self::reasons`] or
    /// [`Self::detail_reasons`].
    pub fn has_reason(&self, reason: &str) -> bool {
        self.reasons
            .iter()
            .chain(&self.detail_reasons)
            .any(|r| r == reason)
    }

    /// Whether Google throttled the request: a 429, a
    /// `RESOURCE_EXHAUSTED` status, or a 403 whose reason is
    /// `rateLimitExceeded`, `userRateLimitExceeded`, `quotaExceeded`
    /// (per-minute quotas such as "Units per minute per user") or
    /// `RATE_LIMIT_EXCEEDED`.
    ///
    /// The daily quota (`dailyLimitExceeded`) is not a rate limit:
    /// waiting a minute does not lift it.
    pub fn is_rate_limited(&self) -> bool {
        if self.status == 429 || self.google_status.as_deref() == Some("RESOURCE_EXHAUSTED") {
            return true;
        }

        self.status == 403
            && [
                "rateLimitExceeded",
                "userRateLimitExceeded",
                "quotaExceeded",
                "RATE_LIMIT_EXCEEDED",
            ]
            .iter()
            .any(|reason| self.has_reason(reason))
    }

    /// Whether the request is worth sending again later: rate limited
    /// ([`Self::is_rate_limited`]) or a transient 5xx.
    pub fn is_retryable(&self) -> bool {
        self.is_rate_limited() || matches!(self.status, 500 | 502 | 503 | 504)
    }

    /// Whether the target of the request does not exist (404).
    pub fn is_not_found(&self) -> bool {
        self.status == 404
    }
}

/// Terminal value of a successful Gmail exchange.
#[derive(Clone, Debug)]
pub struct GmailSendOutput<T> {
    /// The parsed 2xx response body.
    pub response: T,
    /// Whether the server allows reusing the TCP/TLS connection.
    pub keep_alive: bool,
}

/// I/O-free coroutine sending one authorized HTTP request and parsing
/// the JSON response into `T`.
pub struct GmailSend<T> {
    state: State,
    _phantom: PhantomData<T>,
}

impl<T: DeserializeOwned> GmailSend<T> {
    /// Builds a GET request against the given URL.
    pub fn get(auth: &HttpAuthBearer, url: Url) -> Result<Self, GmailSendError> {
        Self::with_method(auth, "GET", url, None, Vec::new())
    }

    /// Builds a DELETE request against the given URL.
    pub fn delete(auth: &HttpAuthBearer, url: Url) -> Result<Self, GmailSendError> {
        Self::with_method(auth, "DELETE", url, None, Vec::new())
    }

    /// Builds a POST request with the given value as JSON body.
    pub fn post_json<B: Serialize>(
        auth: &HttpAuthBearer,
        url: Url,
        body: &B,
    ) -> Result<Self, GmailSendError> {
        let body = serde_json::to_vec(body).map_err(GmailSendError::SerializeRequest)?;
        Self::with_method(auth, "POST", url, Some("application/json"), body)
    }

    /// Builds a PUT request with the given value as JSON body.
    pub fn put_json<B: Serialize>(
        auth: &HttpAuthBearer,
        url: Url,
        body: &B,
    ) -> Result<Self, GmailSendError> {
        let body = serde_json::to_vec(body).map_err(GmailSendError::SerializeRequest)?;
        Self::with_method(auth, "PUT", url, Some("application/json"), body)
    }

    /// Builds a PATCH request with the given value as JSON body.
    pub fn patch_json<B: Serialize>(
        auth: &HttpAuthBearer,
        url: Url,
        body: &B,
    ) -> Result<Self, GmailSendError> {
        let body = serde_json::to_vec(body).map_err(GmailSendError::SerializeRequest)?;
        Self::with_method(auth, "PATCH", url, Some("application/json"), body)
    }

    /// Builds a request with an arbitrary method, content type and body.
    ///
    /// A token holding a control character is refused.
    pub fn with_method(
        auth: &HttpAuthBearer,
        method: &str,
        url: Url,
        content_type: Option<&str>,
        body: Vec<u8>,
    ) -> Result<Self, GmailSendError> {
        let host = url.host_str().unwrap_or("localhost");
        let authorization = gmail_authorization(auth)?;

        let mut request = HttpRequest::get(url.clone())
            .header("Host", host)
            .header("Accept", "application/json")
            .header("Authorization", authorization)
            .body(body);

        if let Some(content_type) = content_type {
            request = request.header("Content-Type", content_type);
        }

        request.method = method.into();

        debug!("prepare request to send");
        trace!("method: {method}");
        trace!("url: {url}");

        Ok(Self {
            state: State::Send(Http11Send::new(request)),
            _phantom: PhantomData,
        })
    }
}

impl<T: DeserializeOwned> GmailCoroutine for GmailSend<T> {
    type Yield = GmailYield;
    type Return = Result<GmailSendOutput<T>, GmailSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
        match &mut self.state {
            State::Send(send) => match send.resume(arg) {
                HttpCoroutineState::Yielded(HttpSendYield::WantsRead) => {
                    GmailCoroutineState::Yielded(GmailYield::WantsRead)
                }
                HttpCoroutineState::Yielded(HttpSendYield::WantsWrite(bytes)) => {
                    GmailCoroutineState::Yielded(GmailYield::WantsWrite(bytes))
                }
                HttpCoroutineState::Yielded(HttpSendYield::WantsRedirect { .. }) => {
                    GmailCoroutineState::Complete(Err(GmailSendError::UnexpectedRedirect))
                }
                HttpCoroutineState::Complete(Err(err)) => {
                    GmailCoroutineState::Complete(Err(err.into()))
                }
                HttpCoroutineState::Complete(Ok(HttpSendOutput {
                    response,
                    remaining,
                    keep_alive,
                })) => {
                    // NOTE: the receive buffer keeps the capacity of the
                    // whole response, free it before parsing the body.
                    drop(remaining);

                    if response.status.is_success() {
                        // NOTE: an empty 2xx body — a DELETE, or a list
                        // endpoint with an empty collection (e.g.
                        // `settings.filters.list`) — is normalised to `{}`.
                        // `GmailNoResponse` ignores it and struct responses
                        // fall back to their `#[serde(default)]` fields;
                        // `null` would fail every struct response with
                        // "invalid type: null".
                        let body = if response.body.is_empty() {
                            b"{}".as_slice()
                        } else {
                            response.body.as_slice()
                        };

                        match serde_json::from_slice::<T>(body) {
                            Ok(response) => GmailCoroutineState::Complete(Ok(GmailSendOutput {
                                response,
                                keep_alive,
                            })),
                            Err(err) => GmailCoroutineState::Complete(Err(
                                GmailSendError::ParseResponse(err),
                            )),
                        }
                    } else {
                        let mut err = GmailApiError::parse(*response.status, &response.body);
                        let delay = response.header("retry-after").and_then(parse_delay);

                        err.retry_after = delay.or(err.retry_after);

                        GmailCoroutineState::Complete(Err(GmailSendError::Api(err)))
                    }
                }
            },
        }
    }
}

enum State {
    Send(Http11Send),
}

#[derive(Debug, Deserialize)]
struct ErrorEnvelope {
    error: ErrorBody,
}

#[derive(Debug, Deserialize)]
struct ErrorBody {
    code: Option<u16>,
    message: Option<String>,
    #[serde(default)]
    errors: Vec<ErrorItem>,
    status: Option<String>,
    #[serde(default)]
    details: Vec<ErrorItem>,
}

/// An item of `error.errors[]` or `error.details[]`, of which only the
/// reason and the retry delay are kept.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ErrorItem {
    reason: Option<String>,
    // NOTE: read loosely, so an unexpected shape cannot fail the envelope.
    retry_delay: Option<Value>,
}

/// Parses Gmail's JSON error envelope, falling back to a summary of the
/// raw body; returns the effective status code and message.
pub fn parse_api_error(http_status: u16, body: &[u8]) -> (u16, String) {
    if let Ok(envelope) = serde_json::from_slice::<ErrorEnvelope>(body) {
        let status = envelope.error.code.unwrap_or(http_status);
        let message = envelope
            .error
            .message
            .filter(|message| !message.trim().is_empty())
            .unwrap_or_else(|| String::from("unknown Gmail API error"));
        return (status, message);
    }

    let message = summarize_body(body);

    if message.is_empty() {
        (http_status, String::from("unknown Gmail API error"))
    } else {
        (http_status, message)
    }
}

/// Maximum length of a summarised error body, in characters.
const SUMMARY_LEN: usize = 200;

/// Number of leading bytes of an error body a summary is made from.
const SUMMARY_SCAN_LEN: usize = 4096;

/// Boils a body that is not an error envelope down to one readable
/// line: the `title` of an HTML page, else its text without markup,
/// whitespace collapsed, capped at [`SUMMARY_LEN`] characters.
fn summarize_body(body: &[u8]) -> String {
    let body = String::from_utf8_lossy(&body[..body.len().min(SUMMARY_SCAN_LEN)]);
    let text = element_text(&body, "title").unwrap_or_else(|| strip_markup(&body));
    let text = text.trim();

    match text.char_indices().nth(SUMMARY_LEN) {
        Some((end, _)) => format!("{}…", &text[..end]),
        None => text.to_string(),
    }
}

/// The trimmed text of the first `<local>` element, when it has any.
fn element_text(body: &str, local: &str) -> Option<String> {
    let open = format!("<{local}>");
    let start = body.find(&open)? + open.len();
    let rest = &body[start..];
    let end = rest.find("</")?;
    let text = rest[..end].trim();

    (!text.is_empty()).then(|| text.to_string())
}

/// Drops every `<...>` tag and collapses the remaining whitespace.
fn strip_markup(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut in_tag = false;

    for ch in body.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if in_tag => continue,
            ch if ch.is_whitespace() => {
                if !out.ends_with(' ') {
                    out.push(' ');
                }
            }
            ch => out.push(ch),
        }
    }

    out
}

/// Reads a delay in seconds: the `Retry-After` header (`120`) or the
/// `retryDelay` of the envelope (`1.5s`). A date reads as none.
pub(crate) fn parse_delay(value: &str) -> Option<Duration> {
    let value = value.trim();
    let value = value.strip_suffix('s').unwrap_or(value);
    let (secs, frac) = value.split_once('.').unwrap_or((value, ""));

    if secs.is_empty() || !secs.chars().chain(frac.chars()).all(|c| c.is_ascii_digit()) {
        return None;
    }

    let secs = secs.parse().ok()?;
    let nanos = frac.bytes().chain([b'0'; 9]).take(9);
    let nanos = nanos.fold(0, |nanos, digit| nanos * 10 + u32::from(digit - b'0'));

    Some(Duration::new(secs, nanos))
}

/// The `Authorization` header value of `auth`, refused when the token
/// holds a control character.
pub(crate) fn gmail_authorization(auth: &HttpAuthBearer) -> Result<String, GmailSendError> {
    let authorization = auth.to_authorization();

    if authorization.chars().any(char::is_control) {
        let message = "Bearer token holds a control character";
        return Err(GmailSendError::InvalidRequest(message.to_string()));
    }

    Ok(authorization)
}

#[cfg(test)]
mod tests {
    use crate::v1::send::{GmailSendError, gmail_api_url};

    #[test]
    fn empty_and_dot_segments_are_refused() {
        for id in ["", ".", ".."] {
            let result = gmail_api_url(&["users", "me", "labels", id]);

            assert!(
                matches!(result, Err(GmailSendError::InvalidRequest(_))),
                "id {id:?} got: {result:?}"
            );
        }
    }

    #[test]
    fn segments_the_url_would_alter_are_refused() {
        for id in [".\t.", "\n..", "Label\t_1", "Label_1\r\n"] {
            let result = gmail_api_url(&["users", "me", "labels", id]);

            assert!(
                matches!(result, Err(GmailSendError::InvalidRequest(_))),
                "id {id:?} got: {result:?}"
            );
        }
    }

    #[test]
    fn each_segment_stays_one_path_segment() {
        let url = gmail_api_url(&["users", "me", "labels", "Label_1?x=1#f/../2"]).unwrap();

        assert_eq!(
            url.as_str(),
            "https://gmail.googleapis.com/gmail/v1/users/me/labels/Label_1%3Fx=1%23f%2F..%2F2"
        );
        assert_eq!(url.query(), None);
        assert_eq!(url.fragment(), None);
    }
}
