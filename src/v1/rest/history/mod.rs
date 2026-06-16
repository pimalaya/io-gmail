//! Gmail mailbox history (`users.history`): the incremental-sync delta
//! since a given `historyId`.
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.history>

mod types;
#[doc(inline)]
pub use types::*;

pub mod list;
