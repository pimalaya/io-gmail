---
cairn: spec
capability: responses
status: current
---

# Responses

How the coroutines read what the server answers: the 2xx bodies and the errors.

### Requirement: A value the crate does not know does not fail the response
Every enum read from a response (`GmailLabelType`, `GmailLabelListVisibility`, `GmailMessageListVisibility`, `GmailDisposition`, `GmailExpungeBehavior`, `GmailPopAccessWindow`, `GmailVerificationStatus`, `GmailSecurityMode`, `GmailFilterSizeComparison`) SHALL read a value outside its known ones as its `Unknown` variant. `Unknown` SHALL NOT be written back: serializing it fails, so a request built from a resource holding it fails with `GmailSendError::SerializeRequest` before anything is sent. An enum only written (a query parameter, a request field) stays closed.

#### Scenario: A label of a new type
- GIVEN a `users.labels.list` answer holding a label whose `type` is `shared`
- WHEN `GmailLabelsList` reads it
- THEN the listing succeeds and the label's type is `Unknown`

#### Scenario: An unknown visibility sent back
- GIVEN a label whose list visibility is `Unknown`
- WHEN `GmailLabelUpdate` is built from it
- THEN it fails with `SerializeRequest`

### Requirement: An API error carries the delay the server asked for
`GmailApiError::retry_after` SHALL hold the `Retry-After` header of the answer when it is a number of seconds, else the `retryDelay` of a `google.rpc.RetryInfo` item of `error.details[]`, else nothing. The date form of the header is not read. In a batch, each call's error SHALL read the header of its own inner answer.

#### Scenario: A header beside a RetryInfo
- GIVEN a 429 answer with `Retry-After: 7` and a `retryDelay` of `1.5s`
- WHEN a coroutine completes with it
- THEN `retry_after` is 7 seconds

### Requirement: A body that is not the error envelope is summarised
When a non-2xx body is not the JSON error envelope, the error message SHALL be made from its first 4096 bytes only: the text of the `title` element when there is one, else the text without markup and with whitespace collapsed, cut at 200 characters and followed by an ellipsis when longer.

#### Scenario: An HTML error page
- GIVEN a 502 answer whose body is an HTML page titled `502 Bad Gateway`
- WHEN the error is built
- THEN its message is `502 Bad Gateway`

### Requirement: The receive buffer is released before the body is parsed
`GmailSend` SHALL drop the bytes buffered past the response, and the capacity that comes with them, before it parses the body, so that reading a response answered with its length never holds more than twice its body: the body beside the value parsed from it. A body answered in several chunks is outside this bound.

#### Scenario: A raw message of 4 MiB
- GIVEN a `users.messages.get` answer whose `raw` field holds 4 MiB, read by 16 KiB or in one piece
- WHEN `GmailMessageGet` completes
- THEN the peak of live bytes it allocated stays under 8 MiB and 64 KiB, where it reached 12 MiB read by 16 KiB before
