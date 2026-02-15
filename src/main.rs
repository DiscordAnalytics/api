use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{App, HttpServer, dev::Service, http, web};
use anyhow::Result;
use tokio::{sync::Mutex, try_join};
use tracing::{Level, info};

use api::{
    app_env,
    config::env::init_env,
    managers::webhook::VotesWebhooksManager,
    repository::Repositories,
    services::Services,
    utils::logger::{LogCode, Logger},
};

#[actix_web::main]
async fn main() -> Result<()> {
    let dev_mode = cfg!(debug_assertions);

    Logger::new()
        .with_level(if dev_mode { Level::DEBUG } else { Level::INFO })
        .init()?;

    info!("[{}] {:-^50}", LogCode::Server, " Starting app ");
    info!(
        "[{}] {}",
        LogCode::Server,
        format!(
            "Running in {} mode",
            if dev_mode {
                "development"
            } else {
                "production"
            }
        )
    );

    init_env()?;
    info!("[{}] {}", LogCode::Server, "Environment initialized");

    let repos = Repositories::init().await?;
    info!(
        "[{}] {}",
        LogCode::Server,
        "Database and R2 storage initialized"
    );

    let services = Services::new(repos.clone());
    info!("[{}] {}", LogCode::Server, "Services initialized");

    let votes_webhooks_manager = web::Data::new(Arc::new(Mutex::new(VotesWebhooksManager::new())));
    info!(
        "[{}] {}",
        LogCode::Server,
        "VotesWebhooksManager initialized"
    );

    let http_server = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&app_env!().client_url)
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH"])
            .allowed_headers(vec![
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
                http::header::CONTENT_TYPE,
            ])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(repos.clone()))
            .app_data(web::Data::new(services.clone()))
            .app_data(votes_webhooks_manager.clone())
            .wrap(cors)
            .wrap_fn(move |req, srv| {
                let fut = srv.call(req);
                Box::pin(async move {
                    let res = fut.await?;

                    info!(
                        "[{}] {} {} {}",
                        LogCode::Request,
                        res.request().method(),
                        res.request().uri(),
                        res.status()
                    );

                    Ok(res)
                })
            })
            .route("/", actix_web::web::get().to(|| async { "Hello, world!" }))
    })
    .bind(("0.0.0.0", app_env!().port))?
    .run();

    info!("[{}] {:-^50}", LogCode::Server, " App started ");
    info!(
        "[{}] {}",
        LogCode::Server,
        format!("Listening on port {}", app_env!().port)
    );
    info!(
        "[{}] {}",
        LogCode::Server,
        format!("Access the API at {}", app_env!().api_url)
    );
    info!(
        "[{}] {}",
        LogCode::Server,
        format!("Access the client at {}", app_env!().client_url)
    );

    try_join!(http_server)?;

    Ok(())
}
