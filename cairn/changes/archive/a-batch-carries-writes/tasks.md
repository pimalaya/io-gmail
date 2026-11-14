---
cairn: tasks
change: a-batch-carries-writes
---

# Tasks

- [x] `GmailBatchRequest::method` and `body`; the writer printing the method, and a body's `Content-Type` and `Content-Length`
- [x] Validation: the five methods, no body on `GET`, no boundary in a body
- [x] `url` on `GmailMessageModify`, `GmailMessageTrash`, `GmailMessageUntrash`, `GmailMessageDelete`; the four batch builders
- [x] Tests: the bytes of a batch holding a `POST` with a body and a `DELETE`; an answer mixing 200, 204 and 404; each refusal
- [x] cairn/spec/batch.md: fold the delta; CHANGELOG; log; status landed
- [ ] The release, left to the owner
