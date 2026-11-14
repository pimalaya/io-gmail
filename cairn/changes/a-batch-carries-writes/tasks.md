---
cairn: tasks
change: a-batch-carries-writes
---

# Tasks

- [ ] `GmailBatchRequest::method` and `body`; the writer printing the method, and a body's `Content-Type` and `Content-Length`
- [ ] Validation: the five methods, no body on `GET`, no boundary in a body
- [ ] `url` on `GmailMessageModify`, `GmailMessageTrash`, `GmailMessageUntrash`, `GmailMessageDelete`; the four batch builders
- [ ] Tests: the bytes of a batch holding a `POST` with a body and a `DELETE`; an answer mixing 200, 204 and 404; each refusal
- [ ] cairn/spec/batch.md: fold the delta; CHANGELOG; the release left to the owner; log; status landed
