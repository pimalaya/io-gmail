# io-gmail architecture

Read the [Pimalaya ARCHITECTURE](https://github.com/pimalaya/.github/blob/master/ARCHITECTURE.md) first: it describes the conventions every Pimalaya repository shares (the sans-I/O coroutine approach, `no_std`, module and error rules, code style, licensing). This document only covers what is specific to io-gmail, and assumes you know that shared context.

If a statement here conflicts with the code, the code wins; please flag it.

## Where io-gmail fits

io-gmail is a **protocol library**: a set of I/O-free coroutines for the [Gmail REST API](https://developers.google.com/gmail/api/reference/rest). It sits one layer above [io-http](https://github.com/pimalaya/io-http) (HTTP/1.1) and [pimalaya-stream](https://github.com/pimalaya/stream) (TCP + TLS), and is consumed by [io-email](https://github.com/pimalaya/io-email) (as the Gmail backend of the shared email domain API) and directly by [himalaya](https://github.com/pimalaya/himalaya) (the protocol-specific `gmail` command). It is the Gmail equivalent of [io-jmap](https://github.com/pimalaya/io-jmap) / [io-imap](https://github.com/pimalaya/io-imap): same shape, different wire protocol (JSON over HTTP rather than IMAP or JMAP).

The crate has two of the three standard layers; there is no CLI:

1. **I/O-free coroutines** (`no_std` core, always present): the whole Gmail REST logic.
2. **Std client** (`client` feature): a blocking driver, `GmailClientStd`, with `connect` gated behind a TLS feature (`rustls-ring` default, `rustls-aws`, `native-tls`).

## The send primitive

Unlike IMAP or JMAP, every Gmail call is an independent HTTP request/response, so io-gmail has a single shared primitive that all domain coroutines delegate to: `send::GmailSend<T>` (`src/send.rs`). It wraps io-http's `Http11Send`, builds the request (the `Authorization` header from the caller's secret, `Accept: application/json`, an optional JSON body), and on completion either deserialises the 2xx body into `T` or parses Gmail's JSON error envelope into `GmailSendError::Api { status, message }`. A 3xx surfaces as `GmailSendError::UnexpectedRedirect` (redirects are not followed). `GmailSend<T>` exposes `get` / `post_json` / `put_json` / `patch_json` / `delete` / `with_method` constructors.

Its terminal value is `GmailSendOutput<T> { response: T, keep_alive: bool }`. `keep_alive` lets a driver reuse the TCP/TLS connection across the many small requests a Gmail session makes. Empty 2xx bodies (DELETE, batch ops) deserialise into the `GmailNoResponse` unit marker.

## The coroutine contract

io-gmail follows the standard Pimalaya coroutine shape with crate-local names (`src/coroutine.rs`):

- trait `GmailCoroutine` with `resume(&mut self, arg: Option<&[u8]>) -> GmailCoroutineState<Self::Yield, Self::Return>`;
- `GmailCoroutineState` is `Yielded(Y)` or `Complete(R)`;
- the standard yield is `GmailYield { WantsRead, WantsWrite(Vec<u8>) }` (Gmail is I/O-only: no clock, randomness or filesystem);
- the `gmail_try!` macro is the coroutine `?`: it forwards `Yielded` and short-circuits `Complete(Err(_))`.

Every domain coroutine is a thin wrapper: a `struct GmailX { state: State }` whose `new(http_auth, user_id, ...)` builds the URL and body and stores a `State::Send(GmailSend<T>)`, and whose `resume` is `let out = gmail_try!(send, arg); Complete(Ok(out))`. `State` has a single `Send` variant and a `fmt::Display` impl, matching the reference template in `src/messages/modify.rs`. Coroutines whose response is empty wrap `GmailSend<GmailNoResponse>`.

## Authentication

io-gmail does no OAuth itself. The caller passes the pre-formatted HTTP `Authorization` header value as a `secrecy::SecretString`, almost always `"Bearer <access-token>"`. Tokens are short-lived; minting and refreshing them is the caller's responsibility (himalaya, for example, reads the bearer from a config command). The base URL is fixed (`GMAIL_API_BASE`, `https://gmail.googleapis.com/gmail/v1/`); the per-mailbox owner is the `user_id` path segment (usually `me`).

## Module layout: one module per Gmail resource domain

The crate is organized to mirror the Gmail API resource tree one-to-one, so a reader who knows the API knows where to look.

```
src/
  lib.rs            crate root: no_std, module declarations, client gated on `client`
  coroutine.rs      GmailCoroutine / GmailCoroutineState / GmailYield + gmail_try!
  send.rs           GmailSend<T>, GmailSendError/Output, GmailNoResponse, parse_api_error, base URLs
  client.rs         (client) GmailClientStd: boxed stream + http_auth + user_id, run/connect
  profile.rs        users.getProfile
  labels/           users.labels       list, get, create, update, delete (+ types)
  messages/         users.messages     list, get, send, modify, trash/untrash, delete,
                                        import, insert, batch_modify, batch_delete,
                                        attachments (messages.attachments.get) (+ types)
  drafts/           users.drafts       list, get, create, update, send, delete (+ types)
  threads/          users.threads      list, get, modify, trash/untrash, delete (+ types)
  history/          users.history      list (+ types)
```

Each domain directory follows the standard module rules: a private `types` submodule re-exported via `#[doc(inline)] pub use types::*;` in `mod.rs`, then one file per verb. Domain types are `Gmail`-prefixed (`GmailMessage`, `GmailLabel`, `GmailDraft`, `GmailThread`, ...) and never re-exported at the crate root; callers use module-qualified paths (`io_gmail::messages::GmailMessage`). JSON shapes are plain serde structs with `#[serde(rename_all = "camelCase")]`; raw RFC 5322 bodies are base64url-encoded via `messages::{encode_raw, decode_raw}`.

Not yet covered: `users.settings.*` (vacation, filters, sendAs, forwardingAddresses, delegates, imap/pop/language/autoForwarding) and `users.watch` / `users.stop` (Pub/Sub push).

## The std client

`GmailClientStd` (`client` feature, `src/client.rs`) wraps a boxed `Read + Write + Send` stream plus the bearer secret and `user_id`. Its generic `run<C, T>(coroutine)` is the blocking driver loop (read on `WantsRead`, write on `WantsWrite`), returning `GmailSendOutput<T>`. It also offers one convenience method per first-class verb (`profile_get`, `labels_list`, `message_get`, `message_send`, ...); newer domains are driven by passing their coroutine to `run`. `connect` (TLS features) opens `gmail.googleapis.com:443` through pimalaya-stream and is the entry point the full client and integration test use.

## Testing

`tests/coroutines.rs` drives coroutines against in-memory HTTP responses (no network) using a `drive` helper, covering parsing, error surfacing and base64url round-trips. `tests/gmail.rs` is an `#[ignore]`d end-to-end test against the live API, gated behind a TLS feature and driven by `GMAIL_ACCESS_TOKEN` (and optional `GMAIL_USER_ID`).
