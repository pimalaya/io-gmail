//! Gmail API v1.
//!
//! `rest` mirrors the REST resource tree (`users.*`); sibling modules
//! add composed coroutines built on the REST ones (e.g. `history_poll`,
//! a poll-based mailbox watcher).

#[cfg(feature = "client")]
pub mod client;
pub mod history_poll;
pub mod query;
pub mod rest;
pub mod send;
