use std::{pin::pin, time::Instant};

use actix_web::{HttpRequest, HttpResponse, web};
use actix_ws::{AggregatedMessage, MessageStream, Session, handle};
use apistos::{
    api_operation,
    web::{ServiceConfig, get, resource, scope},
};
use futures::{
    StreamExt as _,
    future::{Either, select},
};
use tokio::{sync::mpsc, task::spawn_local, time::interval};
use tracing::error;

use crate::{
    domain::error::ApiResult,
    managers::ChatServerHandle,
    utils::{
        constants::{CLIENT_TIMEOUT, HEARTBEAT_INTERVAL},
        logger::LogCode,
    },
};

async fn handle_chat_ws(
    chat_server: ChatServerHandle,
    mut session: Session,
    stream: MessageStream,
) {
    let mut last_hb = Instant::now();
    let mut interval = interval(HEARTBEAT_INTERVAL);

    let (conn_tx, mut conn_rx) = mpsc::unbounded_channel();

    let conn_id = chat_server.connect(conn_tx).await;

    let stream = stream
        .max_frame_size(128 * 1024)
        .aggregate_continuations()
        .max_continuation_size(2 * 1024 * 1024);

    let mut stream = pin!(stream);

    let close_reason = loop {
        let tick = pin!(interval.tick());
        let msg_rx = pin!(conn_rx.recv());
        let messages = pin!(select(stream.next(), msg_rx));

        match select(messages, tick).await {
            Either::Left((Either::Left((Some(Ok(msg)), _)), _)) => match msg {
                AggregatedMessage::Ping(bytes) => {
                    last_hb = Instant::now();
                    session.pong(&bytes).await.expect("Failed to send pong");
                }
                AggregatedMessage::Pong(_) => {
                    last_hb = Instant::now();
                }
                AggregatedMessage::Close(reason) => break reason,
                _ => break None,
            },
            Either::Left((Either::Left((Some(Err(err)), _)), _)) => {
                error!(
                    code = %LogCode::Websocket,
                    error = %err,
                    "Error reading WebSocket message"
                );
                break None;
            }
            // Stream ended
            Either::Left((Either::Left((None, _)), _)) => break None,
            // Send outgoing messages
            Either::Left((Either::Right((Some(chat_msg), _)), _)) => {
                session
                    .text(chat_msg)
                    .await
                    .expect("Failed to send chat message");
            }
            // Channel closed
            Either::Left((Either::Right((None, _)), _)) => break None,
            // Heartbeat tick
            Either::Right((_inst, _)) => {
                if Instant::now().duration_since(last_hb) > CLIENT_TIMEOUT {
                    error!(
                        code = %LogCode::Websocket,
                        conn_id = %conn_id,
                        "Client timeout"
                    );
                    break None;
                }
                let _ = session.ping(b"").await;
            }
        };
    };

    chat_server.disconnect(conn_id).await;
    let _ = session.close(close_reason).await;
}

#[api_operation(
    summary = "WebSocket endpoint for lost connections",
    description = "Establishes a WebSocket connection to receive real-time visitor count updates for the /lost page",
    tag = "WebSocket",
    skip
)]
async fn get_lost(
    req: HttpRequest,
    body: web::Payload,
    chat_server: web::Data<ChatServerHandle>,
) -> ApiResult<HttpResponse> {
    let (response, session, stream) = handle(&req, body)?;

    spawn_local(handle_chat_ws((**chat_server).clone(), session, stream));

    Ok(response)
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(scope("/websocket").service(resource("/lost").route(get().to(get_lost))));
}
