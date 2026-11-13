---
cairn: delta
change: request-and-batch-hardening
---

## ADDED Requirements

### Requirement: The token is one header value
Capability: requests. Every coroutine SHALL refuse a bearer token holding a control character (CR, LF, tab and DEL included) with `GmailSendError::InvalidRequest`, when it is built and before any byte is written. The error message SHALL NOT quote the token.

### Requirement: An inner call is an origin-form request target
Capability: batch (new file). `GmailBatchRequest::get` SHALL keep the path and the query of its URL and drop the fragment. `GmailBatch` SHALL refuse with `GmailSendError::InvalidRequest` a request path that does not start with `/` or that holds `#`, whitespace or a control character.

### Requirement: Reading the answer is bounded by the calls
Capability: batch (new file). `GmailBatch` SHALL read the parts of the answer one at a time, in a single pass over the lines of the body, and SHALL stop once every call has its answer or after twice as many parts as calls. A call whose part lies beyond that bound SHALL be answered `GmailBatchResponseError::Missing`, like a call the answer left out.

## MODIFIED Requirements

### Requirement: An id is one path segment
Capability: requests. Every request URL SHALL be built under `GMAIL_API_BASE` by `gmail_api_url`, which percent-encodes each user id, resource id, label id, attachment id and email address as a single path segment. A value holding `/`, `?`, `#` or `..` among other characters SHALL reach the resource it names verbatim, never another path, a query or a fragment. A value that is empty, `.` or `..` names no resource: `gmail_api_url` SHALL refuse it with `GmailSendError::InvalidRequest`.

## REMOVED Requirements
