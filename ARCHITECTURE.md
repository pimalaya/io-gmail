# io-gmail architecture

Read the [Pimalaya ARCHITECTURE](https://github.com/pimalaya/.github/blob/master/ARCHITECTURE.md) first: it describes the conventions every Pimalaya repository shares (the sans-I/O coroutine approach, `no_std`, module and error rules, code style, licensing). This document only covers what is specific to io-gmail, and assumes you know that shared context.

If a statement here conflicts with the code, the code wins; please flag it.

## Where io-gmail fits

io-gmail is a **protocol library**: a set of I/O-free coroutines for the [Gmail REST API](https://developers.google.com/gmail/api/reference/rest). It sits one layer above [io-http](https://github.com/pimalaya/io-http) (HTTP/1.1) and [pimalaya-stream](https://github.com/pimalaya/stream) (TCP + TLS), and is consumed by [io-email](https://github.com/pimalaya/io-email) (as the Gmail backend of the shared email domain API) and directly by [himalaya](https://github.com/pimalaya/himalaya) (the protocol-specific `gmail` command). It is the Gmail equivalent of [io-jmap](https://github.com/pimalaya/io-jmap) / [io-imap](https://github.com/pimalaya/io-imap): same shape, different wire protocol (JSON over HTTP rather than IMAP or JMAP).

The crate has two of the three standard layers; there is no CLI:

1. **I/O-free coroutines** (`no_std` core, always present): the whole Gmail REST logic.
2. **Std client** (`client` feature): a blocking driver, `GmailClientStd`, with `connect` gated behind a TLS feature (`rustls-ring` default, `rustls-aws`, `native-tls`).

## API versioning: everything lives under `v1`

The Gmail REST API is versioned (`/gmail/v1/`), so the crate is too. The version-agnostic primitives stay at the crate root; everything that is v1-specific lives under `src/v1/`. The day Gmail ships a v2, a sibling `src/v2/` is added without breaking `v1` consumers.

- `src/lib.rs`, `src/coroutine.rs`: crate root, shared across versions.
- `src/v1/`: the v1 surface (`send.rs`, `client.rs`, `history_poll.rs`, and the whole `rest/` tree).

Callers always import through the version, e.g. `io_gmail::v1::rest::labels::GmailLabel`, `io_gmail::v1::client::GmailClientStd`.

## The send primitive

Unlike IMAP or JMAP, every Gmail call is an independent HTTP request/response, so io-gmail has a single shared primitive that all coroutines delegate to: `v1::send::GmailSend<T>` (`src/v1/send.rs`). It wraps io-http's `Http11Send`, builds the request (the `Authorization` header from the caller's secret, `Accept: application/json`, an optional JSON body), and on completion either deserialises the 2xx body into `T` or parses Gmail's JSON error envelope into `GmailSendError::Api { status, message }`. A 3xx surfaces as `GmailSendError::UnexpectedRedirect` (redirects are not followed). `GmailSend<T>` exposes `get` / `post_json` / `put_json` / `patch_json` / `delete` / `with_method` constructors.

Its terminal value is `GmailSendOutput<T> { response: T, keep_alive: bool }`. `keep_alive` lets a driver reuse the TCP/TLS connection across the many small requests a Gmail session makes. Empty 2xx bodies (DELETE, batch ops, stop) deserialise into the `GmailNoResponse` unit marker.

## The coroutine contract

io-gmail follows the standard Pimalaya coroutine shape with crate-local names (`src/coroutine.rs`, version-agnostic):

- trait `GmailCoroutine` with `resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return>`;
- `GmailCoroutineState` is `Yielded(Y)` or `Complete(R)`;
- the standard yield is `GmailYield { WantsRead, WantsWrite(Vec<u8>) }` (a Gmail REST call is I/O-only: no clock, randomness or filesystem);
- the `gmail_try!` macro is the coroutine `?`: it forwards `Yielded` and short-circuits `Complete(Err(_))`.

Every REST coroutine is a thin, single-step wrapper. Because it has exactly one I/O step, it does **not** carry a `State` enum: the struct holds the send directly as `struct GmailX { send: GmailSend<T> }`. `new(http_auth, user_id, ...)` builds the URL and body and stores the `GmailSend<T>`; `resume` is just:

```rust
fn resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return> {
    let out = gmail_try!(&mut self.send, arg);
    debug!("gmail <thing> <verbed>");
    trace!("out: {out:?}");
    GmailCoroutineState::Complete(Ok(out))
}
```

The canonical reference template is the `labels` module (`src/v1/rest/labels/{get,create,delete}.rs`). A multi-variant `State` enum + `fmt::Display` is reserved for genuine multi-step coroutines; in this crate that is only the composite `history_poll` (see below).

### Logging

Each coroutine logs at two levels, via `use log::{debug, trace};`:

- `new()` opens with `debug!("prepare gmail <thing> <op>")` (a static, human-readable lifecycle line), then **one `trace!` per input variable**, each in the form `trace!("var_name: {var_name:?}")`.
- `resume()`, right after the send resolves, does `debug!("gmail <thing> <verbed>")` then `trace!("out: {out:?}")`.

So `debug!` carries the readable lifecycle, `trace!` dumps the raw values; never combine several variables in one `trace!`.

## Authentication

io-gmail does no OAuth itself. The caller passes the pre-formatted HTTP `Authorization` header value as a `secrecy::SecretString`, almost always `"Bearer <access-token>"`. Tokens are short-lived; minting and refreshing them is the caller's responsibility (himalaya, for example, reads the bearer from a config command). The base URL is fixed (`GMAIL_API_BASE`, `https://gmail.googleapis.com/gmail/v1/`); the per-mailbox owner is the `user_id` path segment (usually `me`).

## Module layout: `v1/rest` mirrors the REST tree

`src/v1/rest/` mirrors the Gmail REST reference one-to-one. The whole API hangs off the `users` resource, so that level is flattened away: `v1/rest/` *is* `users`, each sub-resource is a directory, and each method is a file named after the API method in snake_case (`getProfile` -> `get_profile.rs`, `batchModify` -> `batch_modify.rs`, `forwardingAddresses` -> `forwarding_addresses/`). A reader who knows the reference knows where to look.

```
src/
  lib.rs            crate root: no_std, `pub mod coroutine; pub mod v1;`
  coroutine.rs      GmailCoroutine / GmailCoroutineState / GmailYield + gmail_try!
  v1/
    mod.rs
    send.rs         GmailSend<T>, GmailSendError/Output, GmailNoResponse, base URL
    client.rs       (client) GmailClientStd: boxed stream + http_auth + user_id
    history_poll.rs composite watch coroutine (see below)
    rest/
      users/        users.getProfile, users.watch, users.stop (+ types)
      labels/       users.labels       list, get, create, patch, update, delete (+ types)
      messages/     users.messages     list, get, send, modify, trash/untrash, delete,
                                        import, insert, batch_modify, batch_delete,
                                        attachments (messages.attachments.get) (+ types)
      drafts/       users.drafts       list, get, create, update, send, delete (+ types)
      threads/      users.threads      list, get, modify, trash/untrash, delete (+ types)
      history/      users.history      list (+ types)
      settings/     users.settings     get/update {vacation, imap, pop, language,
                                        autoForwarding}; delegates, filters,
                                        forwardingAddresses, sendAs sub-resources (+ types)
```

Every file carries a short `//!` header naming its operation and the REST method in backticks (`//! Get a Gmail label (\`users.labels.get\`).`); each `mod.rs` heads the sub-resource with a one-line summary and the reference URL.

Each directory follows the standard module rules: a private `types` submodule re-exported via `#[doc(inline)] pub use types::*;` in `mod.rs`, then one file per method. `mod.rs` holds only module declarations.

## Types: a faithful, complete mapping of the resource

Domain types are `Gmail`-prefixed (`GmailMessage`, `GmailLabel`, `GmailDraft`, `GmailThread`, `GmailSendAs`, ...), are never re-exported at the crate root (callers use module-qualified paths), and aim to mirror the REST schema fully:

- **Full-resource request bodies.** A method whose body is a resource instance takes the whole resource by reference, not a hand-rolled subset: `labels::create::GmailLabelCreate::new(.., &GmailLabel)`, `messages::send` / `import` / `insert` take `&GmailMessage`, `drafts` create/update/send take `&GmailDraft`, `settings` sendAs/filters/delegates/forwardingAddresses create/update/patch take their resource. Resources are `Default` with `skip_serializing_if` on optional/empty fields, so a partially-filled value serialises cleanly. Methods whose body is a dedicated request object (e.g. `messages.modify`'s add/remove label IDs) keep that small request shape.
- **Enums for enum-valued fields.** Wire strings that the reference documents as enums are typed `Gmail`-prefixed enums (`GmailLabelType`, `GmailMessageListVisibility`, `GmailVerificationStatus`, `GmailDisposition`, ...), each defined in the module it is named after (so `GmailMessageListVisibility` lives in `messages`, even though `labels` references it). Body enums derive serde with `#[serde(rename_all = ...)]`.
- **Query parameters.** List/query methods take a borrowed `*Params<'a>` struct (`GmailDraftsListParams`, `GmailHistoryListParams`) rather than a long positional argument list; the query string is still built by hand (no serde struct->query connector emits the repeated keys Gmail needs). Query-only enums (`GmailMessageFormat`, `GmailInternalDateSource`, `GmailHistoryType`) derive serde rename and are turned into their wire string with `serde_variant::to_variant_name(&value)` at the call site, so the spelling lives only in the rename attribute.
- Raw RFC 5322 message bodies are base64url-encoded via `messages::{encode_raw, decode_raw}`.

## The composite: history polling

`v1/history_poll.rs` is the one multi-step coroutine. `GmailHistoryPoll` is an infinite watch: it baselines the history cursor via `users.getProfile`, then loops `users.history.list` on a timer, yielding the extended `GmailHistoryPollYield { WantsRead, WantsWrite, WantsSleep(Duration), Diff(GmailHistoryDiff) }` and emitting one raw Gmail-native `GmailHistoryDiff` per tick (re-baselining on a 404 expired cursor). It owns a real `State` machine. It deliberately stays Gmail-native: io-email converts each diff into the shared `WatchEvent`, and the std client supplies the actual sleep. This is the polling alternative to `users.watch`/`users.stop` (Pub/Sub push), which exist as plain coroutines for API completeness but are not wired into a watcher.

## The std client

`GmailClientStd` (`client` feature, `src/v1/client.rs`) wraps a boxed `Read + Write + Send` stream plus the bearer secret and `user_id`. Its generic `run<C, T>(coroutine)` is the blocking driver loop (read on `WantsRead`, write on `WantsWrite`), returning `GmailSendOutput<T>`. It offers one convenience method per first-class verb (`profile_get`, `watch`, `stop`, `labels_list`, `message_get`, `message_send`, ...); other coroutines are driven by passing them to `run`. `connect` (TLS features) opens `gmail.googleapis.com:443` through pimalaya-stream and is the entry point the integration test and downstream clients use.

## Testing

`tests/coroutines.rs` drives coroutines against in-memory HTTP responses (no network) using a `drive` helper, covering parsing, error surfacing and base64url round-trips. `tests/gmail.rs` is an `#[ignore]`d end-to-end test against the live API, gated behind a TLS feature and driven by `GMAIL_ACCESS_TOKEN` (and optional `GMAIL_USER_ID`).
