mod client;
mod compiler;
pub mod templates;
mod types;

pub use client::SmtpClient;
pub use templates::{Template, TemplateBuilder, TemplateVars};
pub use types::{MailOptions, MailResult, Recipient};
