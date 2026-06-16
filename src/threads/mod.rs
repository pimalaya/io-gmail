//! Gmail threads (`users.threads`): list, get, modify, trash, untrash,
//! delete.
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.threads>

mod types;
#[doc(inline)]
pub use types::*;

pub mod delete;
pub mod get;
pub mod list;
pub mod modify;
pub mod trash;
