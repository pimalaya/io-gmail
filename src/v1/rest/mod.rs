//! Gmail REST API (`users.*`), flattened: each method is a file and
//! each sub-resource a module, mirroring the reference tree.
//!
//! <https://developers.google.com/gmail/api/reference/rest>

pub mod drafts;
pub mod get_profile;
pub mod history;
pub mod labels;
pub mod messages;
pub mod settings;
pub mod threads;
