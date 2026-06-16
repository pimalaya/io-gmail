#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![no_std]

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
