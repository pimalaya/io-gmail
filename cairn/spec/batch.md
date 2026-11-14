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

### Requirement: An inner call can write
`GmailBatchRequest` SHALL carry a `method`, `GET` by default, and an optional JSON `body`. `GmailBatch` SHALL write each inner call as `<method> <path> HTTP/1.1`, followed for a body by `Content-Type: application/json`, then, for any method but `GET`, its `Content-Length`, 0 without a body, then the body. It SHALL refuse with `GmailSendError::InvalidRequest` a method other than `GET`, `POST`, `PUT`, `PATCH` and `DELETE`, a body on a `GET`, and a body holding the multipart boundary. `GmailBatchRequest::message_modify`, `message_trash`, `message_untrash` and `message_delete` SHALL build the batched form of `users.messages.modify`, `trash`, `untrash` and `delete` from the URL each single call builds, `message_modify` refusing two empty label lists as `GmailMessageModify::new` does.

#### Scenario: A modify and a delete in one batch
- GIVEN a batch holding `message_modify` of `m1` adding `STARRED` and `message_delete` of `m2`
- WHEN it is written
- THEN its parts read `POST /gmail/v1/users/me/messages/m1/modify HTTP/1.1` with the JSON body `GmailMessageModify` sends adding `STARRED`, and `DELETE /gmail/v1/users/me/messages/m2 HTTP/1.1` with `Content-Length: 0` and no body

#### Scenario: An answer per write
- GIVEN that batch
- WHEN Gmail answers `m1` 200 and `m2` 404
- THEN the batch returns `m1` parsed and `m2` as `GmailBatchResponseError::Api` with status 404, `is_not_found` true
