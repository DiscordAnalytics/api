use anyhow::Result;
use tracing::{debug, info, warn};

use crate::{
    app_env,
    domain::models::{Bot, User},
    utils::{
        logger::LogCode,
        mail::{
            MailOptions, MailResult, Recipient, SmtpClient, Template, TemplateBuilder, TemplateVars,
        },
    },
};

#[derive(Clone)]
pub struct MailService {
    client: SmtpClient,
}

impl MailService {
    pub fn new() -> Self {
        Self {
            client: SmtpClient::new().expect("Failed to create SMTP client"),
        }
    }

    pub fn send(
        &self,
        user: &User,
        template: Template,
        vars: impl Into<TemplateVars>,
    ) -> Result<MailResult> {
        if cfg!(debug_assertions) {
            debug!(
                code = %LogCode::Mail,
                user_id = %user.user_id,
                "Skipping email send in debug mode"
            );
            return Ok(MailResult::success());
        }

        if let Some(mail) = &user.mail {
            let recipient = Recipient::new(mail).with_name(&user.username);
            let subject = template.default_subject();
            let html = template.render(vars.into())?;

            let options = MailOptions::new(subject, html).to(recipient);

            self.client.send(options)
        } else {
            warn!(
                code = %LogCode::Mail,
                user_id = %user.user_id,
                "User {} does not have an email address, skipping email sending",
                user.username
            );
            Ok(MailResult::failure())
        }
    }

    pub fn send_bot_configuration_deletion(&self, owner: &User, bot: &Bot) -> Result<MailResult> {
        info!(
            code = %LogCode::Mail,
            bot_id = %bot.bot_id,
            user_id = %owner.user_id,
            "Sending bot configuration deletion email to user {} for bot {}",
            owner.username, bot.username
        );

        let vars = TemplateBuilder::new()
            .var("user_username", &owner.username)
            .var("bot_username", &bot.username)
            .var("bot_id", &bot.bot_id)
            .build();
        self.send(owner, Template::BotConfigurationDeletion, vars)
    }

    pub fn send_bot_configuration_warning(&self, owner: &User, bot: &Bot) -> Result<MailResult> {
        info!(
            code = %LogCode::Mail,
            bot_id = %bot.bot_id,
            user_id = %owner.user_id,
            "Sending bot configuration warning email to user {} for bot {}",
            owner.username, bot.username
        );

        let vars = TemplateBuilder::new()
            .var("user_username", &owner.username)
            .var("bot_username", &bot.username)
            .var("bot_id", &bot.bot_id)
            .build();
        self.send(owner, Template::BotConfigurationWarning, vars)
    }

    pub fn send_bot_deleted_by_admin(
        &self,
        owner: &User,
        bot: &Bot,
        reason: &str,
    ) -> Result<MailResult> {
        info!(
            code = %LogCode::Mail,
            bot_id = %bot.bot_id,
            user_id = %owner.user_id,
            "Sending bot deletion email to user {} for bot {} with reason: {}",
            owner.username, bot.username, reason
        );

        let vars = TemplateBuilder::new()
            .var("user_username", &owner.username)
            .var("bot_username", &bot.username)
            .var("bot_id", &bot.bot_id)
            .var("reason", reason)
            .build();
        self.send(owner, Template::BotDeletedByAdmin, vars)
    }

    pub fn send_bot_inactive_deletion(&self, owner: &User, bot: &Bot) -> Result<MailResult> {
        info!(
            code = %LogCode::Mail,
            bot_id = %bot.bot_id,
            user_id = %owner.user_id,
            "Sending bot inactive deletion email to user {} for bot {}",
            owner.username, bot.username
        );

        let vars = TemplateBuilder::new()
            .var("user_username", &owner.username)
            .var("bot_username", &bot.username)
            .var("bot_id", &bot.bot_id)
            .build();
        self.send(owner, Template::BotInactiveDeletion, vars)
    }

    pub fn send_bot_inactive_warning(&self, owner: &User, bot: &Bot) -> Result<MailResult> {
        info!(
            code = %LogCode::Mail,
            bot_id = %bot.bot_id,
            user_id = %owner.user_id,
            "Sending bot inactive warning email to user {} for bot {}",
            owner.username, bot.username
        );

        let vars = TemplateBuilder::new()
            .var("user_username", &owner.username)
            .var("bot_username", &bot.username)
            .var("bot_id", &bot.bot_id)
            .build();
        self.send(owner, Template::BotInactiveWarning, vars)
    }

    pub fn send_bot_suspended(&self, owner: &User, bot: &Bot, reason: &str) -> Result<MailResult> {
        info!(
            code = %LogCode::Mail,
            bot_id = %bot.bot_id,
            user_id = %owner.user_id,
            "Sending bot suspension email to user {} for bot {} with reason: {}",
            owner.username, bot.username, reason
        );

        let vars = TemplateBuilder::new()
            .var("user_username", &owner.username)
            .var("bot_username", &bot.username)
            .var("bot_id", &bot.bot_id)
            .var("reason", reason)
            .build();
        self.send(owner, Template::BotSuspended, vars)
    }

    pub fn send_bot_token_regen(&self, owner: &User, bot: &Bot) -> Result<MailResult> {
        info!(
            code = %LogCode::Mail,
            bot_id = %bot.bot_id,
            user_id = %owner.user_id,
            "Sending bot token regeneration email to user {} for bot {}",
            owner.username, bot.username
        );

        let vars = TemplateBuilder::new()
            .var("user_username", &owner.username)
            .var("bot_username", &bot.username)
            .var("bot_id", &bot.bot_id)
            .build();
        self.send(owner, Template::BotTokenRegen, vars)
    }

    pub fn send_team_invite(
        &self,
        user: &User,
        owner: &User,
        bot: &Bot,
        invitation_id: impl Into<String>,
    ) -> Result<MailResult> {
        info!(
            code = %LogCode::Mail,
            bot_id = %bot.bot_id,
            user_id = %user.user_id,
            "Sending team invite email to user {} from owner {} for bot {}",
            user.username, owner.username, bot.username
        );

        let vars = TemplateBuilder::new()
            .var("bot_username", &bot.username)
            .var("bot_id", &bot.bot_id)
            .var("owner_username", &owner.username)
            .var("owner_id", &owner.user_id)
            .var(
                "accept_link",
                format!(
                    "{}/invitations/{}",
                    app_env!().client_url,
                    invitation_id.into()
                ),
            )
            .build();

        self.send(user, Template::TeamInvite, vars)
    }

    pub fn send_test_webhook(
        &self,
        owner: &User,
        bot: &Bot,
        provider_name: &str,
        provider_support_url: &str,
    ) -> Result<MailResult> {
        info!(
            code = %LogCode::Mail,
            user_id = %owner.user_id,
            "Sending test webhook email to user {} for bot {} with provider {}",
            owner.username, bot.username, provider_name
        );

        let vars = TemplateBuilder::new()
            .var("user_username", &owner.username)
            .var("bot_username", &bot.username)
            .var("bot_id", &bot.bot_id)
            .var("provider_name", provider_name)
            .var("provider_support_url", provider_support_url)
            .build();

        self.send(owner, Template::TestWebhook, vars)
    }

    pub fn send_user_deleted_by_admin(&self, user: &User) -> Result<MailResult> {
        info!(
            code = %LogCode::Mail,
            user_id = %user.user_id,
            "Sending account deletion email to user {}",
            user.username
        );

        let vars = TemplateBuilder::new()
            .var("user_username", &user.username)
            .build();
        self.send(user, Template::UserDeletedByAdmin, vars)
    }

    pub fn send_user_suspended(&self, user: &User, reason: &str) -> Result<MailResult> {
        info!(
            code = %LogCode::Mail,
            user_id = %user.user_id,
            "Sending account suspension email to user {} with reason: {}",
            user.username, reason
        );

        let vars = TemplateBuilder::new()
            .var("user_username", &user.username)
            .var("reason", reason)
            .build();
        self.send(user, Template::UserSuspended, vars)
    }
}
