---
cairn: tasks
change: request-and-batch-hardening
---

# Tasks

- [x] Reproduce: six failing tests (dot and empty segments, a token with a line break on a coroutine and on a batch, a fragment through `GmailBatchRequest::get` and through a literal path, two stray parts before the answer)
- [x] Refuse empty, `.` and `..` segments in `gmail_api_url`
- [x] Refuse a token holding a control character in `GmailSend::with_method` and `GmailBatch::with_url`
- [x] Return a `Result` from `GmailSend::get`, `GmailSend::delete` and `GmailSend::with_method`, and follow in the 38 request builders
- [x] Drop the fragment in `GmailBatchRequest::get`, refuse `#` in a request path
- [x] Read the answered parts lazily, by line starts, and stop at every call answered or twice as many parts as calls
- [x] Add the `batch` fuzz target and the `fuzz` devShell, run it 60 seconds
- [x] CHANGELOG, spec, log
