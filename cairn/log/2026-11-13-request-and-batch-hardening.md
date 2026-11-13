---
cairn: log
change: request-and-batch-hardening
landed: 2026-11-13
---

# Requests refuse what names nothing, the batch answer is read within bounds

Three findings of the family audit of 2026-11-13 (GMAIL-07, GMAIL-12, GMAIL-10), each reproduced by a failing test first.

An id of `..`, `.` or the empty string built the URL of the parent collection, because the url crate skips dot segments when extending a path: `gmail_api_url` now refuses the three with `InvalidRequest`. A bearer token went into the `Authorization` header as given, so a trailing line feed cut the header block and a CR LF added headers: `GmailSend::with_method` and `GmailBatch::with_url` now refuse a token holding a control character, without quoting it. The check lives here and not in the HTTP library because that dependency is the published 0.6; once its serializer refuses raw control characters in a header value, this one becomes a second line. It costs a break: `GmailSend::get`, `GmailSend::delete` and `GmailSend::with_method` return a `Result`. The alternative, keeping the signatures and failing at the first resume, was set aside: `InvalidRequest` is the error of a request that was never built, and no caller outside the crate uses these three.

In the batch, `GmailBatchRequest::get` kept the fragment of its URL in the inner request line, and a literal path could carry one: the slice now ends at the query and `#` is refused. The answer parser listed every delimiter and every part of the body before matching any, behind a window search whose cost grew with the boundary length. It is now an iterator over line starts, linear in the body, and reading stops when every call has its answer or after twice as many parts as calls (`MAX_PARTS_PER_REQUEST`). Twice is a choice, not a measured need: an honest answer holds exactly one part per call, and the margin leaves room for one stray part per call. What lies beyond reads as `Missing`, which the caller already retries alone.

The new `batch` fuzz target (fuzz/, `fuzz` devShell, modelled on the HTTP library's) answers a batch of one to eight calls with a body assembled from delimiters, close delimiters, `Content-ID` lines, status lines, line breaks and raw bytes. A first run of 60 seconds (386957 executions, nightly 2026-10-31) found nothing. It is not wired into CI, and the devShell cannot be entered through the flake until fuzz/shell.nix is tracked by git.

Capabilities moved: requests (the requirement "An id is one path segment" modified, "The token is one header value" added), batch (new spec file, two requirements).
