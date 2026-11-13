---
cairn: tasks
change: responses-and-errors-hardening
---

# Tasks

- [x] Reproduce: unknown enum values failing a label listing and six settings resources, an HTML error page kept whole as message, a peak of three times a raw message under a counting allocator (tests/memory.rs), and the query functions unable to return an error (compile error)
- [x] Add `Unknown` to the nine enums read from responses, never serialized
- [x] Add `GmailApiError::retry_after`, from the header then from RetryInfo, with accessors on `GmailSendError` and `GmailBatchResponseError`
- [x] Read `Retry-After` in each inner answer of a batch
- [x] Summarise an error body that is not the JSON envelope
- [x] Return a `Result` from `to_query_pairs` and `to_field_pairs`, convert `GmailQueryError` into `InvalidRequest`, follow in the nine call sites
- [x] Drop the receive buffer before parsing the body
- [x] CHANGELOG, spec, log
