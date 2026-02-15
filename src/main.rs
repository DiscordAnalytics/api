mod config;
mod domain;
mod repository;
mod utils;

use actix_web::{App, HttpServer, dev::Service, web};
use anyhow::Result;
use tokio::try_join;
use tracing::{Level, info};

use crate::{
    config::env::init_env,
    repository::Repositories,
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
    info!("[{}] {}", LogCode::Server, "Database connected");

    let http_server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(repos.clone()))
            .route("/", actix_web::web::get().to(|| async { "Hello, world!" }))
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
