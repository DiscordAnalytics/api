use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipient {
    pub email: String,
    pub name: Option<String>,
}

impl Recipient {
    pub fn new(email: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            name: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

impl From<String> for Recipient {
    fn from(email: String) -> Self {
        Self::new(email)
    }
}

impl From<&str> for Recipient {
    fn from(email: &str) -> Self {
        Self::new(email)
    }
}

#[derive(Debug, Clone)]
pub struct MailOptions {
    pub to: Vec<Recipient>,
    pub subject: String,
    pub html_body: String,
}

impl MailOptions {
    pub fn new(subject: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self {
            to: Vec::new(),
            subject: subject.into(),
            html_body: html_body.into(),
        }
    }

    pub fn to(mut self, recipient: impl Into<Recipient>) -> Self {
        self.to.push(recipient.into());
        self
    }
}

#[derive(Debug)]
pub struct MailResult {
    pub success: bool,
}

impl MailResult {
    pub fn success() -> Self {
        Self { success: true }
    }

    pub fn failure() -> Self {
        Self { success: false }
    }
}
