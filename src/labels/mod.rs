//! Gmail labels (`users.labels`): list, get, create, update, delete.
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.labels>

mod types;
#[doc(inline)]
pub use types::*;

pub mod create;
pub mod delete;
pub mod get;
pub mod list;
pub mod update;
