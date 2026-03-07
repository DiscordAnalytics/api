use std::{
    collections::HashMap,
    io,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use ring::rand::{SecureRandom, SystemRandom};
use serde::Serialize;
use tokio::sync::{mpsc, oneshot};
use tracing::info;

use crate::utils::logger::LogCode;

pub type ConnId = u64;
pub type Msg = String;

fn generate_conn_id() -> ConnId {
    let rng = SystemRandom::new();
    let mut bytes = [0u8; 8];
    rng.fill(&mut bytes)
        .expect("Failed to generate random bytes");
    u64::from_le_bytes(bytes)
}

enum Command {
    Connect {
        conn_tx: mpsc::UnboundedSender<Msg>,
        res_tx: oneshot::Sender<ConnId>,
    },
    Disconnect {
        conn_id: ConnId,
    },
}

#[derive(Serialize)]
struct VisitorCount {
    count: usize,
}

pub struct ChatServer {
    sessions: HashMap<ConnId, mpsc::UnboundedSender<Msg>>,
    visitor_count: Arc<AtomicUsize>,
    cmd_rx: mpsc::UnboundedReceiver<Command>,
}

impl ChatServer {
    pub fn new() -> (Self, ChatServerHandle) {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

        (
            Self {
                sessions: HashMap::new(),
                visitor_count: Arc::new(AtomicUsize::new(0)),
                cmd_rx,
            },
            ChatServerHandle { cmd_tx },
        )
    }

    async fn send_system_message(&self, msg: impl Into<Msg>) {
        let system_msg = msg.into();
        for session in self.sessions.values() {
            let _ = session.send(system_msg.clone());
        }
    }

    async fn connect(&mut self, conn_tx: mpsc::UnboundedSender<Msg>) -> ConnId {
        let conn_id = generate_conn_id();
        self.sessions.insert(conn_id, conn_tx);

        let count = self.visitor_count.fetch_add(1, Ordering::SeqCst) + 1;

        info!(
          code = %LogCode::Websocket,
          conn_id = %conn_id,
          visitor_count = %count,
          "New WebSocket connection established"
        );

        self.send_system_message(
            &serde_json::to_string(&VisitorCount { count })
                .expect("Failed to serialize visitor count"),
        )
        .await;

        conn_id
    }

    async fn disconnect(&mut self, conn_id: ConnId) {
        self.sessions.remove(&conn_id);
        let visitor_count = self.visitor_count.fetch_sub(1, Ordering::SeqCst) - 1;

        info!(
          code = %LogCode::Websocket,
          conn_id = %conn_id,
          visitor_count = %visitor_count,
          "WebSocket connection closed"
        );

        self.send_system_message(
            &serde_json::to_string(&VisitorCount {
                count: visitor_count,
            })
            .expect("Failed to serialize visitor count"),
        )
        .await;
    }

    pub async fn run(mut self) -> io::Result<()> {
        info!(
          code = %LogCode::Server,
          "Chat server is running"
        );

        while let Some(cmd) = self.cmd_rx.recv().await {
            match cmd {
                Command::Connect { conn_tx, res_tx } => {
                    let conn_id = self.connect(conn_tx).await;
                    let _ = res_tx.send(conn_id);
                }
                Command::Disconnect { conn_id } => {
                    self.disconnect(conn_id).await;
                }
            }
        }

        info!(
          code = %LogCode::Server,
          "Chat server is shutting down"
        );

        Ok(())
    }
}

#[derive(Clone)]
pub struct ChatServerHandle {
    cmd_tx: mpsc::UnboundedSender<Command>,
}

impl ChatServerHandle {
    pub async fn connect(&self, conn_tx: mpsc::UnboundedSender<Msg>) -> ConnId {
        let (res_tx, res_rx) = oneshot::channel();
        let _ = self.cmd_tx.send(Command::Connect { conn_tx, res_tx });
        res_rx.await.expect("Failed to receive connection ID")
    }

    pub async fn disconnect(&self, conn_id: ConnId) {
        let _ = self.cmd_tx.send(Command::Disconnect { conn_id });
    }
}
