//! User-level Gmail methods (`users`): get profile, watch, stop.
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users>

mod types;
#[doc(inline)]
pub use types::*;

pub mod get_profile;
pub mod stop;
pub mod watch;
