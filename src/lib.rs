#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![no_std]

//! I/O-free coroutines for the Gmail REST API.
//!
//! Each module mirrors a Gmail API resource; see the reference at
//! <https://developers.google.com/gmail/api/reference/rest>.

extern crate alloc;
#[cfg(feature = "client")]
extern crate std;

#[cfg(feature = "client")]
pub mod client;
pub mod coroutine;
pub mod drafts;
pub mod history;
pub mod labels;
pub mod messages;
pub mod profile;
pub mod send;
pub mod settings;
pub mod threads;
pub mod watch;
