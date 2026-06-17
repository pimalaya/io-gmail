//! Gmail filters (`users.settings.filters`): list, get, create, delete.
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.filters>

mod types;
#[doc(inline)]
pub use types::*;

pub mod create;
pub mod delete;
pub mod get;
pub mod list;
