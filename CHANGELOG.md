# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **BREAKING**: Changed the pimalaya-stream dependency to 0.4.

  Its `tls::Tls` moved the trusted certificate from `cert` to `server`, and gained `client`, the certificate presented when the server asks for one (mutual TLS).

- **BREAKING**: Changed the io-http dependency to 0.8.

## [0.7.0] - 2026-11-13

### Added

- Added `gmail_api_url`, building a Gmail API URL from path segments.

- Added a `batch` fuzz target, run from the `fuzz` devShell.

- Added `GmailHistoryPoll::resume_from`, starting the watch from a stored history cursor.

  Its first tick runs at once, without sleeping.

- Added the public field `GmailHistoryPoll::auth`, the bearer of the requests the watch builds from then on.

  Replacing it between ticks lets a watch outlive its access token.

- Added `GmailHistoryPollYield::Expired`, yielded once the watch started over from the current history cursor because its own expired. **Breaking change.**

  It carries the new cursor; the changes before it are lost and the caller resyncs in full. The watch used to start over without a word.

- Added `GmailHistoryPollError::MissingHistoryId`, `TooManyPages`, `RepeatedPageToken` and `Completed`. **Breaking change.**

- Added `GMAIL_HISTORY_POLL_MAX_PAGES`, the number of `users.history.list` pages a tick reads at most.

- Added an `Unknown` variant to `GmailLabelType`, `GmailLabelListVisibility`, `GmailMessageListVisibility`, `GmailDisposition`, `GmailExpungeBehavior`, `GmailPopAccessWindow`, `GmailVerificationStatus`, `GmailSecurityMode` and `GmailFilterSizeComparison`. **Breaking change.**

  A value this crate does not know reads as `Unknown`, where it failed the whole response. Serializing `Unknown` fails, so it is never written back.

- Added `GmailApiError::retry_after`, the delay the server asked for: the `Retry-After` header in its seconds form, else the `retryDelay` of `google.rpc.RetryInfo`. **Breaking change.**

  A `GmailApiError` built as a struct literal needs the new field.

- Added `GmailSendError::retry_after` and `GmailBatchResponseError::retry_after`, the latter reading the header of the call's own answer.

### Changed

- **BREAKING**: Changed the io-http dependency to 0.7.

  `GmailSendError::Send` carries io-http 0.7's `Http11SendError`, which fails with `InvalidRequest`, `UnsupportedTransferCoding` and `TooManyInterimResponses`. The `HttpAuthBearer` the coroutines take has no `Display`.

- Changed `GmailSend::get`, `GmailSend::delete` and `GmailSend::with_method` to return a `Result`. **Breaking change.**

  They fail with `GmailSendError::InvalidRequest` on a bearer token holding a control character.

- Changed `GmailBatch` to read at most twice as many answered parts as calls. **Behaviour change.**

  A call answered beyond that bound reads as `GmailBatchResponseError::Missing`.

- Changed `GmailHistoryPoll` to complete with the error of a failed message fetch. **Behaviour change.**

  It used to report the diff without the message and move its cursor past it. Only a message gone since its listing (404) is still skipped.

- Changed `GmailHistoryPoll` to complete with `GmailHistoryPollError::Completed` when resumed after completion, where it panicked.

- Changed `to_query_pairs` and `to_field_pairs` to return a `Result`. **Breaking change.**

  A value that is not flat fails with `GmailQueryError`, which the coroutines return as `GmailSendError::InvalidRequest`. The request used to go out without its parameters.

- Changed the message of an error whose body is not Gmail's JSON envelope to a summary of at most 200 characters. **Behaviour change.**

  It is the `title` of an HTML page, else its text without markup; it used to be the whole body.

### Fixed

- Fixed ids, label ids, attachment ids, user ids and email addresses going into request paths unencoded: each is now one percent-encoded path segment, so a `?`, `#`, `/` or `..` in it no longer cuts the path or reaches another resource.

- Fixed an empty, `.` or `..` id reaching the parent collection: `gmail_api_url` now refuses it with `GmailSendError::InvalidRequest`.

- Fixed an id holding a tab or a line break reaching the resource named without it, or the parent collection for `.`, tab, `.`: `gmail_api_url` now refuses it with `GmailSendError::InvalidRequest`.

- Fixed a bearer token holding a line break or another control character reaching the `Authorization` header: every coroutine now refuses it with `GmailSendError::InvalidRequest`.

- Fixed `GmailBatchRequest::get` keeping the fragment of its URL in the inner request target.

  `GmailBatch` also refuses a request path holding `#`.

- Fixed the batch answer parser listing every delimiter and part of the body before matching any: it now reads one part at a time, in a single pass over the lines.

- Fixed `GmailHistoryPoll` listing from an empty cursor when the profile carries no history id: it now completes with `GmailHistoryPollError::MissingHistoryId`.

- Fixed `GmailHistoryPoll` following page tokens without end: a tick stops at `GMAIL_HISTORY_POLL_MAX_PAGES` pages or on a repeated token.

- Fixed the receive buffer staying allocated while a response body is parsed.

  A raw message answered with its length is held twice at peak while it is read, where it was held three times.

## [0.6.0] - 2026-11-04

### Added

- Added `GmailRawDecodeError`, returned by `decode_raw` for input outside base64url.

### Changed

- Replaced the base64 crate with [base64-codec](https://crates.io/crates/base64-codec) for `decode_raw` and `encode_raw`.

  It runs on the CPU token the application installs with `simd_cpu::SimdCpu::install_detected`, the portable baseline when none is.

- Changed `decode_raw` to fail with `GmailRawDecodeError` instead of `base64::DecodeError`. **Breaking change.**

- Bumped io-http to 0.6. **Breaking change.**

  The coroutines take its `HttpAuthBearer`, so consumers bump it in step.

- Raised the minimum supported Rust version from 1.88 to 1.89.

## [0.5.0] - 2026-10-07

### Added

- Added `v1::batch::GmailBatch`, sending up to 100 `GET` calls in one `multipart/mixed` request to `https://www.googleapis.com/batch/gmail/v1` (Google advises 50 or fewer for Gmail).

  Calls are `GmailBatchRequest::message_get` (`users.messages.get` with its format and metadata headers) or `GmailBatchRequest::get` on any Gmail API URL. The answer is one `GmailBatchResponse` per call, in the order of the calls whatever the order of the parts, carrying the inner status and either the parsed body or a `GmailBatchResponseError` (`Api` with the whole `GmailApiError`, `ParseResponse`, or `Missing` for a call the answer left out), so throttled calls are retried alone and 404s read as gone. `GmailBatch::with_url` targets another endpoint.

- Added `GmailMessageGet::url`, the `users.messages.get` URL shared with its batched form.

- Added `GmailApiError`, Google's error envelope kept whole: the status, the message, the reasons of `error.errors[]`, the canonical `error.status` and the reasons of `error.details[]`.

  Callers match codes instead of text through `has_reason`, `is_rate_limited` (429, `RESOURCE_EXHAUSTED`, or a 403 for `rateLimitExceeded`, `userRateLimitExceeded`, `quotaExceeded` or `RATE_LIMIT_EXCEEDED`; the daily quota is not a rate limit), `is_retryable` and `is_not_found`.

- Added `GmailSendError::api`, `is_rate_limited`, `is_not_found` and `is_history_expired`, the last one telling a `users.history.list` 404 (the `startHistoryId` is too old or unknown).

### Changed

- Changed `GmailSendError::Api` to carry a `GmailApiError`. **Breaking change.**

  `GmailSendError::Api { status, message }` becomes `GmailSendError::Api(err)`, with `err.status` and `err.message`.

- Changed `GmailSendError::is_retryable` to also hold for a 403 rate limit, beside 429 and 5xx. **Behaviour change.**

## [0.4.0] - 2026-09-29

### Added

- Added `GmailClientStdConnectOptions::proxy`, tunnelling the connection through a SOCKS5 or HTTP proxy.

## [0.3.1] - 2026-09-28

### Fixed

- Fixed `no_std` builds pulling in `std` ([#2]).

  The `serde_variant` dependency was dropped, enum query parameters now go through the in-crate query serializer.

## [0.3.0] - 2026-08-15

### Changed

- Bumped pimalaya-stream to 0.3. **Behaviour change.**

  Consumers must move with it, since the `Tls` type comes from that version. The transport now retries a spurious `EAGAIN` for a minute before failing with `TimedOut`, and arms a read deadline at connect time.

- Bumped io-http to 0.5.
- Raised the minimum supported Rust version from 1.87 to 1.88.

## [0.2.2] - 2026-07-25

### Added

- Added an optional `schemars` feature deriving `JsonSchema` on the REST output types.

  Off by default and still `no_std`. It covers the profile, label, message, draft, thread, history and settings types, with their list responses.

## [0.2.1] - 2026-07-25

### Fixed

- Fixed struct responses failing to parse on an empty 2xx body.

  An empty body is now read as `{}` instead of `null`, so `GmailNoResponse` and empty list responses parse.

## [0.2.0] - 2026-07-16

### Changed

- Moved each REST type into its resource module, dropping the internal `types` submodules.

  Entity types keep their path. Operation-specific types moved into their operation module, e.g. `rest::labels::list::GmailLabelsListResponse`, `rest::users::get_profile::GmailProfile` and `rest::users::watch::GmailWatchRequest`.

## [0.1.0] - 2026-07-15

### Added

- Added the I/O-free coroutine core for the Gmail REST API v1.
- Added the full `v1::rest` surface: users, labels, messages, drafts, threads, history and settings.
- Added `v1::history_poll::GmailHistoryPoll`, a poll-based mailbox watch.
- Added `GmailClientStd`, a std blocking client behind the `client` feature.

[unreleased]: https://github.com/pimalaya/io-gmail/compare/v0.7.0..HEAD
[0.7.0]: https://github.com/pimalaya/io-gmail/compare/v0.6.0..v0.7.0
[0.6.0]: https://github.com/pimalaya/io-gmail/compare/v0.5.0..v0.6.0
[0.5.0]: https://github.com/pimalaya/io-gmail/compare/v0.4.0..v0.5.0
[0.4.0]: https://github.com/pimalaya/io-gmail/compare/v0.3.1..v0.4.0
[0.3.1]: https://github.com/pimalaya/io-gmail/compare/v0.3.0..v0.3.1
[0.3.0]: https://github.com/pimalaya/io-gmail/compare/v0.2.2..v0.3.0
[0.2.2]: https://github.com/pimalaya/io-gmail/compare/v0.2.1..v0.2.2
[0.2.1]: https://github.com/pimalaya/io-gmail/compare/v0.2.0..v0.2.1
[0.2.0]: https://github.com/pimalaya/io-gmail/compare/v0.1.0..v0.2.0
[0.1.0]: https://github.com/pimalaya/io-gmail/compare/root..v0.1.0

[#2]: https://github.com/pimalaya/io-gmail/issues/2
