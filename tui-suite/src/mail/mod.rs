pub mod compose;
pub mod gmail;
pub mod mime;

pub use compose::ComposeState;
pub use gmail::GmailProvider;
pub use mime::{build_mime_message, encode_for_gmail, EmailMessage};
