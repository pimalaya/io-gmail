---
cairn: log
change: a-batch-carries-writes
landed: 2026-11-14
---

# A batch carries writes

`GmailBatch` sent only `GET` inner calls, so himalaya changed, trashed and deleted Gmail messages one request each, stopping at the first failure. Himalaya's a-result-carries-its-errors wants a per-message answer for each write, which Gmail's batch endpoint gives in one round trip; `users.messages.batchModify` and `batchDelete` answer the whole set at once and say nothing per message.

`GmailBatchRequest` gained `method` (`GET` by default, through a hand-written `Default`) and `body: Option<Vec<u8>>`. The writer prints `<method> <path> HTTP/1.1`, for a body `Content-Type: application/json`, for any method but `GET` a `Content-Length` (0 without a body), then the body. `GmailBatch` refuses with `InvalidRequest` a method other than `GET`, `POST`, `PUT`, `PATCH` and `DELETE` (matched exactly, so lowercase is refused too), a body on a `GET`, and a body holding the multipart boundary. A body on a `DELETE` is allowed, as the proposal asked.

`GmailMessageModify`, `GmailMessageTrash`, `GmailMessageUntrash` and `GmailMessageDelete` gained a `url`, like `GmailMessageGet::url`, and `GmailBatchRequest` gained `message_modify`, `message_trash`, `message_untrash` and `message_delete` on top of them. The modify body and its check of two empty label lists moved into a crate-private `GmailMessageModifyRequest::new`, which both the single call and the batch builder use, so the batched body is byte for byte the one `GmailMessageModify` sends (a test compares the two). Trash, untrash and delete carry `Content-Length: 0` and no body, as io-http sends the single calls, since Google answers a bodiless `POST` without a length 411. No live test was run.

The answers are read as before: an empty 2xx part, the one `messages.delete` answers, reads as `{}`.

The fuzz target was updated for the new fields. It does not build today for a reason older than this change: fuzz/Cargo.toml still pins io-http 0.7 while the crate is on 0.8.

Capabilities moved: batch (the requirement "An inner call can write" added).
