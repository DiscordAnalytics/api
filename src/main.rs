mod config;
mod utils;

use std::io;

use actix_web::{App, HttpServer};
use log::{LevelFilter, info};
use tokio::try_join;

use crate::{
    config::env::init_env,
    utils::logger::{LogCode, Logger},
};

#[actix_web::main]
async fn main() -> io::Result<()> {
    let dev_mode = cfg!(debug_assertions);

    Logger::new()
        .level(if dev_mode {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        })
        .log_to_file(true)
        .log_dir("logs".into())
        .dev_mode(dev_mode)
        .init()
        .expect("Failed to initialize logger");

    info!("[{}] {:-^50}", LogCode::Server, " Starting app ");

    init_env().expect("Failed to initialize environment variables");

    let http_server = HttpServer::new(move || {
        App::new().route("/", actix_web::web::get().to(|| async { "Hello, world!" }))
    })
    .bind(("0.0.0.0", app_env!().port))?
    .run();

    try_join!(http_server)?;

    Ok(())
}
