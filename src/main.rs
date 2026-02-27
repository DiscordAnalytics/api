use std::{net::Ipv4Addr, sync::Arc};

use actix_cors::Cors;
use actix_web::{App, HttpServer, http, rt, web};
use anyhow::Result;
use apistos::app::OpenApiWrapper;
use tokio::{
    spawn,
    sync::Mutex,
    time::{Duration, interval},
    try_join,
};
use tracing::{Level, error, info};
use tracing_actix_web::TracingLogger;

use api::{
    api::{middleware::AuthMiddleware, routes},
    app_env,
    config::env::init_env,
    managers::{ChatServer, VotesWebhooksManager},
    openapi::build_spec,
    repository::Repositories,
    services::Services,
    utils::logger::{LogCode, Logger},
};

#[actix_web::main]
async fn main() -> Result<()> {
    let dev_mode = cfg!(debug_assertions);

    init_env().expect("Failed to initialize environment variables");

    Logger::new()
        .with_level(if dev_mode { Level::DEBUG } else { Level::INFO })
        .init()
        .expect("Failed to initialize logger");

    info!(
        code = %LogCode::Server,
        "Starting app",
    );
    info!(
        code = %LogCode::Server,
        "Running in {} mode",
        if dev_mode {
            "development"
        } else {
            "production"
        }
    );

    let repos = Repositories::init().await?;
    info!(
        code = %LogCode::Server,
        "Repositories initialized",
    );

    let repos_clone = repos.clone();
    rt::spawn(async move {
        let repos_clone = repos_clone.clone();
        let mut interval = interval(Duration::from_secs(3600));

        loop {
            interval.tick().await;

            match repos_clone.sessions.delete_expired().await {
                Ok(deleted_count) => info!(
                    code = %LogCode::Server,
                    deleted_count = %deleted_count,
                    "Deleted expired sessions",
                ),
                Err(e) => error!(
                    code = %LogCode::Server,
                    error = %e,
                    "Failed to delete expired sessions"
                ),
            }
        }
    });

    let services = Services::new(repos.clone());
    info!(
        code = %LogCode::Server,
        "Services initialized",
    );

    let votes_webhooks_manager = web::Data::new(Arc::new(Mutex::new(VotesWebhooksManager::new())));
    info!(
        code = %LogCode::Server,
        "VotesWebhooksManager initialized",
    );

    let (chat_server, chat_server_handle) = ChatServer::new();
    let chat_server = spawn(chat_server.run());
    let chat_server_handle = web::Data::new(chat_server_handle);
    info!(
        code = %LogCode::Server,
        "ChatServer initialized",
    );

    let http_server = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&app_env!().client_url)
            .allowed_methods(vec!["GET", "POST", "DELETE", "PATCH"])
            .allowed_headers(vec![
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
                http::header::CONTENT_TYPE,
            ])
            .supports_credentials()
            .max_age(3600);

        let spec = build_spec();

        App::new()
            .document(spec)
            .app_data(web::Data::new(repos.clone()))
            .app_data(web::Data::new(services.clone()))
            .app_data(votes_webhooks_manager.clone())
            .app_data(chat_server_handle.clone())
            .wrap(TracingLogger::default())
            .wrap(cors)
            .wrap(AuthMiddleware)
            .configure(routes::configure)
            .build("/openapi.json")
    })
    .bind((Ipv4Addr::UNSPECIFIED, app_env!().port))?
    .run();

    info!(
        code = %LogCode::Server,
        "App started",
    );
    info!(
        code = %LogCode::Server,
        "Listening on port {}",
        app_env!().port,
    );
    info!(
        code = %LogCode::Server,
        "Access the API at {}",
        app_env!().api_url,
    );
    info!(
        code = %LogCode::Server,
        "Access the client at {}",
        app_env!().client_url,
    );

    try_join!(http_server, async { chat_server.await? })?;

    Ok(())
}
