//! Gmail settings (`users.settings`): vacation, IMAP, POP, language,
//! auto-forwarding, filters, forwarding addresses, delegates and
//! send-as aliases.
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings>

mod types;
#[doc(inline)]
pub use types::*;

pub mod delegates;
pub mod filters;
pub mod forwarding_addresses;
pub mod send_as;

pub mod get_auto_forwarding;
pub mod get_imap;
pub mod get_language;
pub mod get_pop;
pub mod get_vacation;
pub mod update_auto_forwarding;
pub mod update_imap;
pub mod update_language;
pub mod update_pop;
pub mod update_vacation;
