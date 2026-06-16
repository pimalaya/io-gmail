//! Gmail settings (`users.settings`): vacation, IMAP, POP, language,
//! auto-forwarding, filters, forwarding addresses, delegates and
//! send-as aliases.
//!
//! <https://developers.google.com/gmail/api/reference/rest/v1/users.settings>

pub mod autoforwarding;
pub mod delegates;
pub mod filters;
pub mod forwarding_addresses;
pub mod imap;
pub mod language;
pub mod pop;
pub mod sendas;
pub mod vacation;
