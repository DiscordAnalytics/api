use crate::{app_env, utils::discord::DiscordEmbed};

pub enum NotificationType {
    #[cfg(not(debug_assertions))]
    BotConfigurationDeletion {
        bot_username: String,
        bot_id: String,
    },
    #[cfg(not(debug_assertions))]
    BotConfigurationWarning {
        bot_username: String,
        bot_id: String,
    },
    BotDeletedByAdmin {
        bot_username: String,
        bot_id: String,
        reason: String,
    },
    BotSuspended {
        bot_username: String,
        bot_id: String,
        reason: String,
    },
    BotTokenRegen {
        bot_username: String,
        bot_id: String,
    },
    TeamInvite {
        bot_username: String,
        owner_username: String,
        invitation_id: String,
    },
    TestWebhook {
        bot_username: String,
        bot_id: String,
        provider: String,
        provider_url: String,
    },
    UserDeletedByAdmin {
        username: String,
        user_id: String,
    },
    UserSuspended {
        username: String,
        user_id: String,
        reason: String,
    },
}

impl NotificationType {
    pub fn to_embed(&self) -> DiscordEmbed {
        match self {
            #[cfg(not(debug_assertions))]
            NotificationType::BotConfigurationDeletion {
                bot_username,
                bot_id,
            } => DiscordEmbed::new()
                .title("Bot Configuration Deletion")
                .description(format!(
                    "Your bot **{}** has been deleted due to not being configured.\n\n\
                    If you believe this was a mistake or would like more information, please contact [support](https://discordanalytics.xyz/support).",
                    bot_username
                ))
                .color(0xE74C3C) // Red
                .field("Bot", format!("{} ({})", bot_username, bot_id), false)
                .footer("Discord Analytics"),

            #[cfg(not(debug_assertions))]
            NotificationType::BotConfigurationWarning {
              bot_username,
              bot_id
            } => DiscordEmbed::new()
              .title("Bot Configuration Warning")
              .description(format!(
                "Your bot **{}** is not yet configured.\n\n\
                Please configure your bot in the next 24 hours before we delete it from our platform.\n\n\
                You can follow our [documentation](https://discordanalytics.xyz/docs/get-started/installation) to get started",
                bot_username
              ))
              .color(0xF1C40F) // Yellow
              .field("Bot", format!("{} ({})", bot_username, bot_id), false)
              .footer("Discord Analytics"),


            NotificationType::BotDeletedByAdmin {
                bot_username,
                bot_id,
                reason,
            } => DiscordEmbed::new()
                .title("Bot Deleted by Admin")
                .description(format!(
                    "Your bot **{}** has been deleted by an administrator.\n\n\
                    **Reason:** {}\n\n\
                    If you believe this was a mistake or would like more information, please contact support.",
                    bot_username, reason
                ))
                .color(0xE74C3C) // Red
                .field("Bot", format!("{} ({})", bot_username, bot_id), false)
                .footer("Discord Analytics"),

            NotificationType::BotSuspended {
                bot_username,
                bot_id,
                reason,
            } => DiscordEmbed::new()
                .title("Bot Suspended")
                .description(format!(
                    "Your bot **{}** has been suspended.\n\n\
                    **Reason:** {}\n\n\
                    During the suspension period, your bot will not be able to access the API. \
                    Please contact support for more information or to resolve the issue.",
                    bot_username, reason
                ))
                .color(0xE74C3C) // Red
                .field("Bot", format!("{} ({})", bot_username, bot_id), false)
                .footer("Discord Analytics"),

            NotificationType::BotTokenRegen {
                bot_username,
                bot_id,
            } => DiscordEmbed::new()
                .title("Bot Token Regenerated")
                .description(format!(
                    "The API token for your bot **{}** has been regenerated.\n\n\
                    **Important:** Your old token has been invalidated and will no longer work. \
                    Make sure to update your bot's configuration with the new token.",
                    bot_username
                ))
                .color(0xFFC107) // Amber/Warning
                .field("Bot", format!("{} ({})", bot_username, bot_id), false)
                .footer("Discord Analytics"),

            NotificationType::TeamInvite {
                bot_username,
                owner_username,
                invitation_id,
            } => DiscordEmbed::new()
                .title("Team Invitation")
                .description(format!(
                    "You've been invited to join the team for bot **{}** by **{}**.",
                    bot_username, owner_username
                ))
                .color(0x5865F2) // Discord Blurple
                .field(
                    "Accept Invitation",
                    format!(
                        "[Click here]({}/invitation/{})",
                        app_env!().client_url,
                        invitation_id
                    ),
                    false,
                )
                .footer("Discord Analytics"),

            NotificationType::TestWebhook {
                bot_username,
                bot_id,
                provider,
                provider_url,
            } => DiscordEmbed::new()
                .title("Test Webhook Received")
                .description(format!(
                    "We successfully received your test webhook for bot **{}**!",
                    bot_username
                ))
                .color(0x43B581) // Green
                .field("Bot", format!("{} ({})", bot_username, bot_id), false)
                .field("Provider", provider, true)
                .field("Support URL", provider_url, true)
                .footer("Discord Analytics"),

            NotificationType::UserDeletedByAdmin {
                username,
                user_id,
            } => DiscordEmbed::new()
                .title("Account Deleted by Admin")
                .description(format!(
                    "Your account **{}** has been deleted by an administrator.\n\n\
                    All of your bots and data have been permanently removed. \n\n\
                    If you believe this was a mistake or would like more information, please contact support.",
                    username
                ))
                .color(0xE74C3C) // Red
                .field("User", format!("{} ({})", username, user_id), false)
                .footer("Discord Analytics"),

            NotificationType::UserSuspended {
                username,
                user_id,
                reason,
            } => DiscordEmbed::new()
                .title("Account Suspended")
                .description(format!(
                    "Your account **{}** has been suspended.\n\n\
                    **Reason:** {}\n\n\
                    During the suspension period, you will not be able to access the API or manage your bots. \
                    Please contact support for more information or to resolve the issue.",
                    username, reason
                ))
                .color(0xE74C3C) // Red
                .field("User", format!("{} ({})", username, user_id), false)
                .footer("Discord Analytics"),
        }
    }
}

pub struct DiscordNotification;

impl DiscordNotification {
    pub fn create(notification_type: NotificationType) -> Vec<DiscordEmbed> {
        vec![notification_type.to_embed()]
    }
}
