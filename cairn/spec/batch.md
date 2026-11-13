---
cairn: spec
capability: batch
status: current
---

# Batch

How `GmailBatch` writes the inner calls of a batch and reads their answers.

### Requirement: An inner call is an origin-form request target
`GmailBatchRequest::get` SHALL keep the path and the query of its URL and drop the fragment. `GmailBatch` SHALL refuse with `GmailSendError::InvalidRequest` a request path that does not start with `/` or that holds `#`, whitespace or a control character.

#### Scenario: A URL carrying a fragment
- GIVEN the URL `https://gmail.googleapis.com/gmail/v1/users/me/labels?fields=labels#frag`
- WHEN `GmailBatchRequest::get` is built for it
- THEN its path is `/gmail/v1/users/me/labels?fields=labels`

### Requirement: Reading the answer is bounded by the calls
`GmailBatch` SHALL read the parts of the answer one at a time, in a single pass over the lines of the body, and SHALL stop once every call has its answer or after twice as many parts as calls. A call whose part lies beyond that bound SHALL be answered `GmailBatchResponseError::Missing`, like a call the answer left out.

#### Scenario: Two stray parts before the answer
- GIVEN a batch of one call
- WHEN the answer holds two parts with unknown `Content-ID`s, then the part of the call
- THEN the call is answered `Missing`, with no status

#### Scenario: One stray part before the answer
- GIVEN a batch of one call
- WHEN the answer holds one part with an unknown `Content-ID`, then the part of the call
- THEN the call gets its answer
