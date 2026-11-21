//! Gmail API v1.
//!
//! `rest` mirrors the REST resource tree (`users.*`); sibling modules
//! add coroutines beyond it: `batch`, several REST calls in one HTTP
//! request, and `history_poll`, a poll-based mailbox watcher.

#[cfg(feature = "client")]
pub mod batch;
#[cfg(feature = "std")]
pub mod client;
#[cfg(feature = "client")]
pub mod history_poll;
#[cfg(feature = "client")]
pub mod query;
pub mod rest;
#[cfg(feature = "client")]
pub mod send;
