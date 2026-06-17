//! Gmail forwarding addresses (`users.settings.forwardingAddresses`):
//! list, get, create, delete.
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings.forwardingAddresses>

mod types;
#[doc(inline)]
pub use types::*;

pub mod create;
pub mod delete;
pub mod get;
pub mod list;
