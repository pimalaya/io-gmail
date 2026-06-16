//! Gmail messages (`users.messages`), including attachments
//! (`users.messages.attachments`).
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.messages>

mod types;
#[doc(inline)]
pub use types::*;

pub mod attachments;
pub mod batch_delete;
pub mod batch_modify;
pub mod delete;
pub mod get;
pub mod import;
pub mod insert;
pub mod list;
pub mod modify;
pub mod send;
pub mod trash;
