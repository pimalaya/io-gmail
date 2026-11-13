---
cairn: change
id: responses-and-errors-hardening
status: landed
created: 2026-11-13
---

# Read what the server adds, tell what it asked, refuse what cannot be sent

## Why

Four findings of the family audit of 2026-11-13, on how the crate reads answers and builds queries.

- GMAIL-06: the enums read from responses were closed. One value added on the server side (a third label type, a new visibility) failed `serde_json::from_slice` for the whole response, so `users.labels.list` became unusable for that mailbox until a release, although the caller wanted ids and names.
- GMAIL-08: `GmailApiError` kept the status, the message and the reasons, and dropped the delay the server asks for, whether in the `Retry-After` header or in a `google.rpc.RetryInfo` detail, leaving the caller to guess. Separately, a body that is not the JSON envelope became the message whole: an HTML error page of any size.
- GMAIL-09: `to_query_pairs` ended with `unwrap_or_default` and `to_field_pairs` with `Err(_) => Vec::new()`. A value the serializer cannot flatten produced no pair at all, the ones already gathered included, and the request went out unfiltered. No params struct shipped today is affected (all are flat), but both functions are public.
- GMAIL-05, the part that is this crate's: `GmailSend::resume` matched `HttpSendOutput { response, keep_alive, .. }`, so the receive buffer of the HTTP layer, emptied but with the capacity of the whole response, lived until the end of the arm, through the parsing of the body. A raw message was held three times at peak.

## What

- An `Unknown` variant marked `#[serde(other, skip_serializing)]` on the nine enums read from responses. Reading never fails on a new value; writing `Unknown` fails, on purpose: leaving the field out would let a full update reset it on the server without a word, and no literal would be right to send. Enums that are only written stay closed.
- `GmailApiError::retry_after`, filled from the header in its seconds form, else from RetryInfo, on the same rules and with the same `parse_delay` as the Tasks crate of the family; `GmailSendError::retry_after` and `GmailBatchResponseError::retry_after` read it, and a batch reads the header of each inner answer. The fallback message is a summary: the `title` of the page, else the text without markup, 200 characters at most, made from the first 4096 bytes of the body.
- `to_query_pairs` and `to_field_pairs` return a `Result`; `GmailQueryError` converts into `GmailSendError::InvalidRequest`, and the nine call sites propagate it.
- `GmailSend::resume` binds `remaining` and drops it before parsing.

## Breaks

- Nine enums gain a variant (exhaustive matches).
- `GmailApiError` gains a public field (struct literals).
- `to_query_pairs` and `to_field_pairs` return a `Result`.

## Not in this change

The other half of GMAIL-05: the HTTP layer still copies the body out of its buffer, and send, insert and import still go through the JSON `raw` field; both need the HTTP library or new upload coroutines. `decode_raw` still needs its input beside its output.
