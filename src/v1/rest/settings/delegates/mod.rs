//! Gmail delegates (`users.settings.delegates`): list, get, create,
//! delete.
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.delegates>

mod types;
#[doc(inline)]
pub use types::*;

pub mod create;
pub mod delete;
pub mod get;
pub mod list;
