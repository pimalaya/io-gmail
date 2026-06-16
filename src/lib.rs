#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

//! I/O-free coroutines for the Gmail REST API.
//!
//! Each module mirrors a Gmail API resource; see the reference at
//! <https://developers.google.com/gmail/api/reference/rest>.

extern crate alloc;
#[cfg(feature = "client")]
extern crate std;

pub mod coroutine;
pub mod v1;
