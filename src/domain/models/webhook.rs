use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

#[derive(Clone, Debug, Serialize, PartialEq)]
pub enum Provider {
    TopGG,
    DiscordList,
    DiscordsCom,
    BotListMe,
    DBList,
    DiscordPlace,
    Test,
}

impl Provider {
    pub fn to_str(&self) -> &'static str {
        match self {
            Provider::TopGG => "topgg",
            Provider::DiscordList => "discordlist",
            Provider::DiscordsCom => "discordscom",
            Provider::BotListMe => "botlistme",
            Provider::DBList => "dblist",
            Provider::DiscordPlace => "discordplace",
            _ => "test",
        }
    }

    pub fn parse_str(provider: &str) -> Self {
        match provider {
            "topgg" => Provider::TopGG,
            "discordlist" => Provider::DiscordList,
            "discordscom" => Provider::DiscordsCom,
            "botlistme" => Provider::BotListMe,
            "dblist" => Provider::DBList,
            "discordplace" => Provider::DiscordPlace,
            _ => Provider::Test,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookData {
    pub bot_id: String,
    pub voter_id: String,
    pub provider: Provider,
    pub date: DateTime<Utc>,
    pub raw_data: Option<Value>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookSendData<'a> {
    pub bot_id: &'a str,
    pub voter_id: &'a str,
    pub provider: &'a str,
    pub date: DateTime<Utc>,
    pub raw_data: Option<&'a Value>,
    pub content: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Webhook {
    pub webhook_url: String,
    pub webhook_secret: Option<String>,
    pub data: WebhookData,
    pub try_count: u8,
}

impl PartialEq for Webhook {
    fn eq(&self, other: &Self) -> bool {
        self.data.bot_id == other.data.bot_id
            && self.data.voter_id == other.data.voter_id
            && self.data.provider == other.data.provider
            && self.data.date == other.data.date
            && self.data.raw_data == other.data.raw_data
    }
}
