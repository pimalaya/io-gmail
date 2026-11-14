---
cairn: change
id: a-batch-carries-writes
status: active
created: 2026-11-14
---

# A batch carries writes

## Why

`GmailBatch` sends only `GET` inner calls: `GmailBatchRequest` holds an id and a path, and the writer prints `GET <path> HTTP/1.1` for each. Himalaya therefore changes, trashes and deletes messages one HTTP request each, stopping at the first failure. Himalaya is making each write report the messages it could not change, next to the ones it did (pimalaya/himalaya, cairn/changes/a-result-carries-its-errors). Gmail's batch endpoint answers each inner call with its own status, up to 100 calls per request (developers.google.com/workspace/gmail/api/guides/batch), so one batch gives the same per-message answers in a single round trip.

`users.messages.batchModify` and `batchDelete` are no substitute. They answer an empty body for the whole set, and `batchDelete` gives "no guarantees that messages were not already deleted or even existed". The caller learns nothing per message.

## What

- `GmailBatchRequest` gains `method`, `GET` by default, and `body: Option<Vec<u8>>`, a JSON body. The writer prints the method, and for a body it adds `Content-Type: application/json` and its `Content-Length` before the body.
- `GmailBatch` refuses with `GmailSendError::InvalidRequest`:
  - a method other than `GET`, `POST`, `PUT`, `PATCH` and `DELETE`;
  - a body on a `GET`;
  - a body holding the multipart boundary.
- Builders reuse each single call's URL. `GmailMessageModify`, `GmailMessageTrash`, `GmailMessageUntrash` and `GmailMessageDelete` gain a `url` like `GmailMessageGet::url`, and `GmailBatchRequest` gains `message_modify`, `message_trash`, `message_untrash` and `message_delete`.
  - `message_modify` refuses two empty label lists, as `GmailMessageModify::new` does.
- The answers are read as today. An empty 2xx body, which `messages.delete` answers, already reads as `{}`. A batch mixing calls of several kinds reads its answers as `serde_json::Value`.

## Scope / non-goals

- The batch keeps its bounds of 1 to 100 calls and its single pass over the answer.
- No call beyond the four message writes gets a builder. `GmailBatchRequest` with a method, a path and a body reaches any other.

## Decisions

- **Breaking.** `GmailBatchRequest` gains public fields, so struct literals naming every field break. This is a 0.x minor release, and the owner decides the version.
