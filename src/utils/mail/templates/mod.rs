use std::collections::HashMap;

use anyhow::Result;

use crate::utils::mail::compiler::compile_mjml;

pub type TemplateVars = HashMap<String, String>;

#[derive(Debug, Clone, Copy)]
pub enum Template {
    BotDeletedByAdmin,
    BotSuspended,
    BotTokenRegen,
    TeamInvite,
    TestWebhook,
    UserDeletedByAdmin,
    UserSuspended,
}

impl Template {
    pub fn mjml(&self) -> &'static str {
        match self {
            Template::BotDeletedByAdmin => include_str!("bot_deleted_by_admin.mjml"),
            Template::BotSuspended => include_str!("bot_suspended.mjml"),
            Template::BotTokenRegen => include_str!("bot_token_regen.mjml"),
            Template::TeamInvite => include_str!("team_invite.mjml"),
            Template::TestWebhook => include_str!("test_webhook.mjml"),
            Template::UserDeletedByAdmin => include_str!("user_deleted_by_admin.mjml"),
            Template::UserSuspended => include_str!("user_suspended.mjml"),
        }
    }

    pub fn default_subject(&self) -> &'static str {
        match self {
            Template::BotDeletedByAdmin => "Your bot has been deleted by an administrator",
            Template::BotSuspended => "Your bot has been suspended",
            Template::BotTokenRegen => "Your bot token has been regenerated",
            Template::TeamInvite => "You've been invited to join a team",
            Template::TestWebhook => "Test Webhook Notification",
            Template::UserDeletedByAdmin => "Your account has been deleted by an administrator",
            Template::UserSuspended => "Account Suspension Notice",
        }
    }

    pub fn render(&self, vars: TemplateVars) -> Result<String> {
        let mut mjml = self.mjml().to_string();

        for (key, value) in vars {
            let placeholder = format!("{{{{{}}}}}", key);
            mjml = mjml.replace(&placeholder, &value);
        }

        compile_mjml(&mjml)
    }
}

pub struct TemplateBuilder {
    vars: TemplateVars,
}

impl TemplateBuilder {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    pub fn var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.vars.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> TemplateVars {
        self.vars
    }
}

impl Default for TemplateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_all_templates() {
        let templates = [
            Template::BotDeletedByAdmin,
            Template::BotSuspended,
            Template::BotTokenRegen,
            Template::TeamInvite,
            Template::TestWebhook,
            Template::UserDeletedByAdmin,
            Template::UserSuspended,
        ];

        for template in templates {
            let vars = TemplateBuilder::new().build();

            let rendered = template.render(vars.clone());
            assert!(
                rendered.is_ok(),
                "Failed to render template: {:?}",
                template
            );
        }
    }

    #[test]
    fn test_var_replacement() {
        let template = Template::BotSuspended;
        let vars = TemplateBuilder::new()
            .var("user_username", "testuser")
            .var("bot_username", "testbot")
            .var("bot_id", "12345")
            .var("reason", "Violation of terms")
            .build();

        let rendered = template.render(vars).expect("Failed to render template");
        assert!(rendered.contains("testuser"));
        assert!(rendered.contains("testbot"));
        assert!(rendered.contains("12345"));
        assert!(rendered.contains("Violation of terms"));
    }
}
