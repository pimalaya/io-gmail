---
cairn: change
id: request-and-batch-hardening
status: landed
created: 2026-11-13
---

# Refuse what cannot be a request, bound what is read from a batch

## Why

Three defects found by the family audit of 2026-11-13, all on what the crate writes or on the one parser of server bytes it owns.

- GMAIL-07: `gmail_api_url` hands its segments to `PathSegmentsMut::extend`, which skips `.` and `..` and keeps an empty one as a trailing slash. An id of `..` therefore built the URL of the parent collection (`users.labels.get` became `users.labels.list`, whose answer parses as a default label), against the requirement that an id reaches the resource it names.
- GMAIL-12: the bearer token went into the `Authorization` header as given, and the HTTP serializer writes header values raw. A token read with its trailing line feed ended the header block early, and one holding CR LF added headers.
- GMAIL-10: `GmailBatchRequest::get` sliced its URL from the path to the end, fragment included, so the inner request line carried `#frag`. On the answer side `split_parts` recorded every delimiter and every part of the body in two vectors before any was matched, behind a window-by-window search, although at most one part per call is ever used. The parser had no fuzz target.

## What

- `gmail_api_url` refuses an empty, `.` or `..` segment with `GmailSendError::InvalidRequest`.
- The token is checked where it becomes a header, in this crate, since the HTTP dependency comes from crates.io: `GmailSend::with_method` and `GmailBatch::with_url` refuse a token holding a control character with `InvalidRequest`. `GmailSend::get`, `GmailSend::delete` and `GmailSend::with_method` return a `Result` for it (breaking); the 38 request builders calling them already returned one. The token is refused, not trimmed: the caller owns the secret and learns its source is wrong.
- `GmailBatchRequest::get` slices up to the end of the query, and `GmailBatch` refuses a request path holding `#`, which covers a request built as a literal.
- The answer is read one part at a time by an iterator scanning line starts, so the cost is linear in the body whatever the boundary, and reading stops once every call has its answer or after twice as many parts as calls. The rest reads as `Missing`, which is retryable.
- A `batch` fuzz target under fuzz/, exposed as the `fuzz` devShell, runs the coroutine against a multipart body assembled from fuzzed pieces. It is not wired into CI.
