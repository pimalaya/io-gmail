# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Fixed `no_std` builds by dropping the `serde_variant` dependency, which pulls in `std`. Enum query parameters now go through the in-crate query serializer ([#2]).

## [0.3.0] - 2026-08-15

### Changed

- Bumped pimalaya-stream to 0.3, which drops the `sasl` module it no longer owns and whose `Read` and `Write` retry a stream reporting it is not ready. **Behaviour change.**

  The `Tls` type this crate takes comes from that version, so a consumer must move with it. A blocking socket is not supposed to report `EAGAIN`, yet callers saw one surface mid-exchange and end the exchange with a bare `Resource temporarily unavailable (os error 35)`, macOS especially. The transport now retries such a failure for a minute before giving up with a `TimedOut` naming the budget, and arms a socket read deadline at connect time so a server going silent on a healthy connection stops blocking the caller forever.

- Bumped io-http to 0.5.
- Raised the minimum supported Rust version from 1.87 to 1.88, following pimalaya-stream and io-http.

## [0.2.2] - 2026-07-25

### Added

- Added an optional `schemars` feature deriving `schemars::JsonSchema` on the serializable REST output types, so downstream tools can generate JSON Schemas describing Gmail command output.

  The feature is off by default and stays `no_std`: it pulls only schemars' `derive` (not `std`). It covers the getProfile, label, message (payload, parts, headers), draft, thread and history response types, plus the settings filters, forwardingAddresses, delegates and sendAs types together with their list responses.

## [0.2.1] - 2026-07-25

### Fixed

- Fixed struct responses failing to parse when Gmail returns an empty 2xx body.

  A DELETE, or a list endpoint whose collection is empty (e.g. `settings.filters.list`, `settings.forwardingAddresses.list`), returns no body; `GmailSend` normalised that to `null`, which failed every struct response with `invalid type: null`. It is now normalised to `{}`, so `GmailNoResponse` still ignores it and list responses fall back to their `#[serde(default)]` fields.

## [0.2.0] - 2026-07-16

### Changed

- Reorganised the REST resource types so each lives in its resource module directly, dropping the internal `types` submodules and per-type files together with their flattened re-exports.

  Entity and value-object types keep their existing path (`rest::labels::GmailLabel`, `rest::messages::GmailMessage`, `rest::settings::GmailImapSettings` and so on). Operation-specific companions moved into their operation module: `rest::labels::GmailLabelsListResponse` is now `rest::labels::list::GmailLabelsListResponse`, `rest::users::GmailProfile` is now `rest::users::get_profile::GmailProfile`, and `rest::users::GmailWatchRequest`, `GmailWatchResponse` and `GmailLabelFilterBehavior` now live under `rest::users::watch`.

## [0.1.0] - 2026-07-15

### Added

- Added the I/O-free coroutine core for the Gmail REST API v1: the `GmailCoroutine` contract, the shared `GmailSend` HTTP/JSON primitive parsing Gmail's error envelope (redirects are never followed), and a `no_std` query-pair serializer for list parameters.
- Added the full `v1::rest` surface mirroring the Gmail REST reference: users (getProfile, watch, stop), labels, messages (including import, insert, batch operations and attachments), drafts, threads, history, and settings (imap, pop, vacation, language, autoForwarding, delegates, filters, forwardingAddresses, sendAs).
- Added `v1::history_poll::GmailHistoryPoll`, an infinite poll-based mailbox watch composing `users.getProfile` and `users.history.list`, emitting one Gmail-native diff per tick and re-baselining on an expired history cursor.
- Added `GmailClientStd` (`client` feature): a std blocking client with one convenience method per first-class verb, a generic `run` loop for the other coroutines, and a `connect` constructor opening gmail.googleapis.com through pimalaya-stream (`rustls-ring` default, `rustls-aws`, `native-tls`).

[unreleased]: https://github.com/pimalaya/io-gmail/compare/v0.3.0..HEAD
[0.3.0]: https://github.com/pimalaya/io-gmail/compare/v0.2.2..v0.3.0
[0.2.2]: https://github.com/pimalaya/io-gmail/compare/v0.2.1..v0.2.2
[0.2.1]: https://github.com/pimalaya/io-gmail/compare/v0.2.0..v0.2.1
[0.2.0]: https://github.com/pimalaya/io-gmail/compare/v0.1.0..v0.2.0
[0.1.0]: https://github.com/pimalaya/io-gmail/compare/root..v0.1.0

[#2]: https://github.com/pimalaya/io-gmail/issues/2
