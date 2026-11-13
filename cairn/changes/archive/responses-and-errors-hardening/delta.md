---
cairn: delta
change: responses-and-errors-hardening
---

## ADDED Requirements

### Requirement: Query parameters are written whole or the request is refused
Capability: requests. `to_query_pairs` and `to_field_pairs` SHALL fail with `GmailQueryError` on a value they cannot flatten (anything but a flat struct of scalars, options and sequences), and SHALL NOT return the pairs gathered before the failure. A coroutine SHALL return that failure as `GmailSendError::InvalidRequest` and SHALL NOT send the request without its parameters.

### Requirement: A value the crate does not know does not fail the response
Capability: responses (new file). Every enum read from a response (`GmailLabelType`, `GmailLabelListVisibility`, `GmailMessageListVisibility`, `GmailDisposition`, `GmailExpungeBehavior`, `GmailPopAccessWindow`, `GmailVerificationStatus`, `GmailSecurityMode`, `GmailFilterSizeComparison`) SHALL read a value outside its known ones as its `Unknown` variant. `Unknown` SHALL NOT be written back: serializing it fails, so a request built from a resource holding it fails with `GmailSendError::SerializeRequest` before anything is sent. An enum only written (a query parameter, a request field) stays closed.

### Requirement: An API error carries the delay the server asked for
Capability: responses (new file). `GmailApiError::retry_after` SHALL hold the `Retry-After` header of the answer when it is a number of seconds, else the `retryDelay` of a `google.rpc.RetryInfo` item of `error.details[]`, else nothing. The date form of the header is not read. In a batch, each call's error SHALL read the header of its own inner answer.

### Requirement: A body that is not the error envelope is summarised
Capability: responses (new file). When a non-2xx body is not the JSON error envelope, the error message SHALL be made from its first 4096 bytes only: the text of the `title` element when there is one, else the text without markup and with whitespace collapsed, cut at 200 characters and followed by an ellipsis when longer.

### Requirement: The receive buffer is released before the body is parsed
Capability: responses (new file). `GmailSend` SHALL drop the bytes buffered past the response, and the capacity that comes with them, before it parses the body, so that reading a response never holds more than the buffer beside the body, then the body beside the value parsed from it.

## MODIFIED Requirements

## REMOVED Requirements
