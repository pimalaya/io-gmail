# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[unreleased]: https://github.com/pimalaya/io-gmail/compare/v0.5.0..HEAD
[0.5.0]: https://github.com/pimalaya/io-gmail/compare/v0.4.0..v0.5.0
[0.4.0]: https://github.com/pimalaya/io-gmail/compare/v0.3.1..v0.4.0
[0.3.1]: https://github.com/pimalaya/io-gmail/compare/v0.3.0..v0.3.1
[0.3.0]: https://github.com/pimalaya/io-gmail/compare/v0.2.2..v0.3.0
[0.2.2]: https://github.com/pimalaya/io-gmail/compare/v0.2.1..v0.2.2
[0.2.1]: https://github.com/pimalaya/io-gmail/compare/v0.2.0..v0.2.1
[0.2.0]: https://github.com/pimalaya/io-gmail/compare/v0.1.0..v0.2.0
[0.1.0]: https://github.com/pimalaya/io-gmail/compare/root..v0.1.0

[#2]: https://github.com/pimalaya/io-gmail/issues/2
