pub mod schemas;

use apistos::{
    info::{Contact, Info},
    paths::ExternalDocumentation,
    server::Server,
    spec::Spec,
    tag::Tag,
};

use crate::app_env;

pub fn build_spec() -> Spec {
    Spec {
        info: Info {
            title: "Discord Analytics API".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: Some(
                "Official Discord Analytics API for bot statistics and analytics".to_string(),
            ),
            terms_of_service: Some(
                "https://discordanalytics.xyz/docs/legals/terms.html".to_string(),
            ),
            contact: Some(Contact {
                name: Some("Discord Analytics".to_string()),
                url: Some("https://discordanalytics.xyz".to_string()),
                email: Some("contact@discordanalytics.xyz".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        },
        servers: vec![Server {
            url: if cfg!(debug_assertions) {
                app_env!().api_url.to_owned()
            } else {
                "https://discordanalytics.xyz".to_string()
            },
            description: Some("Base URL for the Discord Analytics API".to_string()),
            ..Default::default()
        }],
        external_docs: Some(ExternalDocumentation {
            description: Some("Discord Analytics Documentation".to_string()),
            url: "https://discordanalytics.xyz/docs".to_string(),
            ..Default::default()
        }),
        tags: vec![
            Tag {
                name: "Achievements".to_string(),
                description: Some("Endpoints for managing and retrieving achievements".to_string()),
                ..Default::default()
            },
            Tag {
                name: "Analytics".to_string(),
                description: Some("Endpoints for retrieving analytics data".to_string()),
                ..Default::default()
            },
            Tag {
                name: "Bots".to_string(),
                description: Some(
                    "Endpoints for managing and retrieving bot information".to_string(),
                ),
                ..Default::default()
            },
            Tag {
                name: "Health".to_string(),
                description: Some("Endpoints related to API health and status".to_string()),
                ..Default::default()
            },
            Tag {
                name: "Webhooks".to_string(),
                description: Some("Endpoints for managing webhooks".to_string()),
                ..Default::default()
            },
        ],

        ..Default::default()
    }
}
