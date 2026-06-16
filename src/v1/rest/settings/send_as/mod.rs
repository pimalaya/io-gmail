//! Gmail send-as aliases (`users.settings.sendAs`): list, get, create,
//! update, patch, delete, verify.
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.sendAs>

mod types;
#[doc(inline)]
pub use types::*;

pub mod create;
pub mod delete;
pub mod get;
pub mod list;
pub mod patch;
pub mod update;
pub mod verify;
