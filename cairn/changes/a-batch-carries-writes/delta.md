---
cairn: delta
change: a-batch-carries-writes
---

# Delta

## ADDED Requirements

### Requirement: An inner call can write
(batch) `GmailBatchRequest` SHALL carry a `method`, `GET` by default, and an optional JSON `body`. `GmailBatch` SHALL write each inner call as `<method> <path> HTTP/1.1`, followed for a body by `Content-Type: application/json`, its `Content-Length` and the body. It SHALL refuse with `GmailSendError::InvalidRequest` a method other than `GET`, `POST`, `PUT`, `PATCH` and `DELETE`, a body on a `GET`, and a body holding the multipart boundary. `GmailBatchRequest::message_modify`, `message_trash`, `message_untrash` and `message_delete` SHALL build the batched form of `users.messages.modify`, `trash`, `untrash` and `delete` from the URL each single call builds, `message_modify` refusing two empty label lists as `GmailMessageModify::new` does.

#### Scenario: A modify and a delete in one batch
- GIVEN a batch holding `message_modify` of `m1` adding `STARRED` and `message_delete` of `m2`
- WHEN it is written
- THEN its parts read `POST /gmail/v1/users/me/messages/m1/modify HTTP/1.1` with the JSON body `GmailMessageModify` sends adding `STARRED`, and `DELETE /gmail/v1/users/me/messages/m2 HTTP/1.1` with no body

#### Scenario: An answer per write
- GIVEN that batch
- WHEN Gmail answers `m1` 200 and `m2` 404
- THEN the batch returns `m1` parsed and `m2` as `GmailBatchResponseError::Api` with status 404, `is_not_found` true

## MODIFIED Requirements

None.

## REMOVED Requirements

None.
