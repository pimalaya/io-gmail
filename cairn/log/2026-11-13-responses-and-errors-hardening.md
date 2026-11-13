---
cairn: log
change: responses-and-errors-hardening
landed: 2026-11-13
---

# Unknown enum values are read, errors carry their delay, query errors are returned, the receive buffer is freed

Four findings of the family audit of 2026-11-13 (GMAIL-06, GMAIL-08, GMAIL-09 and the local part of GMAIL-05).

The nine enums read from responses take an `Unknown` variant, `#[serde(other, skip_serializing)]`. A listing no longer fails on a value the server added. Writing `Unknown` back fails with `SerializeRequest`: the two other ways were to leave the field out, which a full update turns into a reset on the server, or to send a literal the server never defined. A caller holding such a resource patches the fields it means to change. With the `schemars` feature the variant shows in the schema of the six enums that derive it, as the constant `unknown`.

`GmailApiError::retry_after` is the `Retry-After` header in seconds, else the `retryDelay` of RetryInfo; the header wins because it is the transport's own word. `parse_delay` and the shape of the envelope item are those of the Tasks crate, so the two read the same answers the same way; that crate also carries `keep_alive` on its error, which is left out here until the std client's handling of a failed exchange is decided. The summary of a non-JSON body is the People crate's, with one difference: it reads the first 4096 bytes only, so that a page of any size costs the same.

`to_query_pairs` and `to_field_pairs` return their error. It had no red test at run time, since no params struct of the crate can fail: the test was a compile error on the old signature.

For the memory half, tests/memory.rs installs a counting allocator and reads a 4 MiB raw message in one piece: the peak of live bytes was 12583116 before, three times the message (the receive buffer, the body copied from it, the string parsed from the body), and stays under the 10 MiB bound of the test after, since the buffer is dropped before the parse. The batch coroutine already released it at the end of its `let`. What remains is outside this crate or needs new coroutines: the body copy in the HTTP layer, the JSON `raw` path of send, insert and import, and `decode_raw` holding its input beside its output.

Capabilities moved: requests (one requirement added), responses (new spec file, four requirements).
