//! Gmail API v1.
//!
//! `rest` mirrors the REST resource tree (`users.*`); sibling modules
//! add coroutines beyond it: `batch`, several REST calls in one HTTP
//! request, and `history_poll`, a poll-based mailbox watcher.

pub mod batch;
#[cfg(feature = "client")]
pub mod client;
pub mod history_poll;
pub mod query;
pub mod rest;
pub mod send;
