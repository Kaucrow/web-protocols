//! A WebSockets server.

use std::{
    collections::HashMap,
    io,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use uuid::Uuid;
use tokio::sync::{mpsc, oneshot};
use crate::{ConnId, Msg};

/// A command received by the [`Server`].
#[derive(Debug)]
enum Command {
    Connect {
        conn_tx: mpsc::UnboundedSender<Msg>,
        res_tx: oneshot::Sender<ConnId>,
    },

    Disconnect {
        conn: ConnId,
    },

    HandleFrame {
        conn: ConnId,
        frame: String,
        res_tx: oneshot::Sender<()>,
    },
}

/// A server.
///
/// Contains the logic of how connections with each other plus room management.
///
/// Call and spawn [`run`](Self::run) to start processing commands.
#[derive(Debug)]
pub struct Server {
    /// Map of connection IDs to their message receivers.
    sessions: HashMap<ConnId, mpsc::UnboundedSender<Msg>>,

    /// Tracks total number of historical connections established.
    visitor_count: Arc<AtomicUsize>,

    /// Command receiver.
    cmd_rx: mpsc::UnboundedReceiver<Command>,
}

impl Server {
    pub fn new() -> (Self, ServerHandle) {
        // Server commands tx and rx, for interaction between Server and ServerHandler
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

        (
            Self {
                sessions: HashMap::new(),
                visitor_count: Arc::new(AtomicUsize::new(0)),
                cmd_rx,
            },
            ServerHandle { cmd_tx },
        )
    }

    fn get_conn_tx(&self, conn_id: ConnId) -> Result<&mpsc::UnboundedSender<Msg>, ()> {
        if let Some(conn_id) = self.sessions.get(&conn_id) {
            Ok(conn_id)
        } else {
            Err(())
        }
    }

    /// Send message to users in a room.
    ///
    /// `skip` is used to prevent messages triggered by a connection also being received by it.
    async fn send_system_message(&self, skip: Option<ConnId>, msg: impl Into<Msg>) {
        let msg = msg.into();

        for session in &self.sessions {
            if let Some(skip) = skip {
                if *session.0 == skip {
                    continue;
                }
            }

            let tx = session.1;

            // errors if client disconnected abruptly and hasn't been timed-out yet
            let _ = tx.send(msg.clone());
        }
    }

    /// Register new session and assign unique ID to this session
    async fn connect(&mut self, conn_tx: mpsc::UnboundedSender<Msg>) -> ConnId {
        // Notify all users
        self.send_system_message(None, "Someone joined").await;

        // Register session with random connection ID
        let id = Uuid::new_v4();
        self.sessions.insert(id, conn_tx);

        let count = self.visitor_count.fetch_add(1, Ordering::SeqCst);
        self.send_system_message(None, format!("Total visitors {count}"))
            .await;

        // Send id back
        id
    }

    async fn handle_frame(&self, conn: ConnId, frame: String) {
        tracing::event!(target: "backend", tracing::Level::DEBUG, "Client sent frame: {}", frame);
    }

    /// Unregister connection from room map and broadcast disconnection message.
    async fn disconnect(&mut self, conn_id: ConnId) {
        println!("Someone disconnected");

        // Remove sender
        if self.sessions.remove(&conn_id).is_none() {
            println!("Tried to remove an unexistent session.");
        };

        // Send message to other users
        self.send_system_message(None, "Someone disconnected").await;
    }

    pub async fn run(mut self) -> io::Result<()> {
        while let Some(cmd) = self.cmd_rx.recv().await {
            match cmd {
                Command::Connect { conn_tx, res_tx } => {
                    let conn_id = self.connect(conn_tx).await;
                    let _ = res_tx.send(conn_id);
                }

                Command::Disconnect { conn } => {
                    self.disconnect(conn).await;
                }

                Command::HandleFrame { conn, frame, res_tx} => {
                    self.handle_frame(conn, frame).await;
                    let _ = res_tx.send(());
                }

                _ => unimplemented!()
            }
        }

        Ok(())
    }
}

/// Handle and command sender for server.
///
/// Reduces boilerplate of setting up response channels in WebSocket handlers.
#[derive(Debug, Clone)]
pub struct ServerHandle {
    cmd_tx: mpsc::UnboundedSender<Command>,
}

impl ServerHandle {
    /// Register client message sender and obtain connection ID.
    pub async fn connect(&self, conn_tx: mpsc::UnboundedSender<Msg>) -> ConnId {
        let (res_tx, res_rx) = oneshot::channel();

        // Unwrap: server should not have been dropped
        self.cmd_tx
            .send(Command::Connect { conn_tx, res_tx })
            .unwrap();

        // Unwrap: server does not drop out response channel
        res_rx.await.unwrap()
    }

    pub async fn handle_frame(&self, conn: ConnId, frame: String) {
        let (res_tx, res_rx) = oneshot::channel();

        self.cmd_tx
            .send(Command::HandleFrame { conn, frame, res_tx })
            .unwrap();

        res_rx.await.unwrap();
    }

    /// Unregister message sender and broadcast disconnection message to all users.
    pub fn disconnect(&self, conn: ConnId) {
        // Unwrap: server should not have been dropped
        self.cmd_tx.send(Command::Disconnect { conn }).unwrap();
    }
}
