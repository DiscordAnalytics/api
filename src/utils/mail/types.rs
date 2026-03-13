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
    pub text_body: Option<String>,
}

impl MailOptions {
    pub fn new(subject: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self {
            to: Vec::new(),
            subject: subject.into(),
            html_body: html_body.into(),
            text_body: None,
        }
    }

    pub fn to(mut self, recipient: impl Into<Recipient>) -> Self {
        self.to.push(recipient.into());
        self
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text_body = Some(text.into());
        self
    }
}

#[derive(Debug)]
pub struct MailResult {
    pub success: bool,
    pub message_id: Option<String>,
    pub error: Option<String>,
}

impl MailResult {
    pub fn success(message_id: impl Into<String>) -> Self {
        Self {
            success: true,
            message_id: Some(message_id.into()),
            error: None,
        }
    }

    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            message_id: None,
            error: Some(error.into()),
        }
    }
}
