use anyhow::Result;
use lettre::{
    Message, SmtpTransport, Transport,
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use tracing::{error, info};

use crate::{app_env, utils::logger::LogCode};

use super::types::{MailOptions, MailResult};

#[derive(Clone)]
pub struct SmtpClient {
    transport: SmtpTransport,
    from: Mailbox,
}

impl SmtpClient {
    pub fn new() -> Result<Self> {
        let env = app_env!();

        let creds = Credentials::new(env.smtp_user.clone(), env.smtp_password.clone());

        let transport = SmtpTransport::relay(&env.smtp)?.credentials(creds).build();

        let from = env.smtp_mail.parse()?;

        Ok(Self { transport, from })
    }

    pub fn send(&self, options: MailOptions) -> Result<MailResult> {
        let mut email = Message::builder()
            .from(self.from.clone())
            .subject(options.subject);

        for recipient in options.to {
            let mailbox = if let Some(name) = recipient.name {
                Mailbox::new(Some(name), recipient.email.parse()?)
            } else {
                recipient.email.parse()?
            };
            email = email.to(mailbox);
        }

        let email = email
            .header(ContentType::TEXT_HTML)
            .body(options.html_body)?;

        match self.transport.send(&email) {
            Ok(response) => {
                info!(
                    code = %LogCode::Mail,
                    "Email sent successfully"
                );
                Ok(MailResult::success(format!("{:?}", response)))
            }
            Err(e) => {
                error!(
                    code = %LogCode::Mail,
                    error = %e,
                    "Failed to send email"
                );
                Ok(MailResult::failure(e.to_string()))
            }
        }
    }
}

impl Default for SmtpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create SMTP client")
    }
}
