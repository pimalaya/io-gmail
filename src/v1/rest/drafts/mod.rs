//! Gmail drafts (`users.drafts`): list, get, create, update, send, delete.
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.drafts>

mod types;
#[doc(inline)]
pub use types::*;

pub mod create;
pub mod delete;
pub mod get;
pub mod list;
pub mod send;
pub mod update;
