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
use dyn_fmt::AsStrFormatExt;
use crate::{
    ConnId, Msg,
    handler::ClientInfo,
};

/// A command received by the [`Server`].
#[derive(Debug)]
enum Command {
    Connect {
        conn_tx: mpsc::UnboundedSender<Msg>,
        res_tx: oneshot::Sender<ConnId>,
    },

    Disconnect {
        client: ClientInfo,
        conn: ConnId,
    },

    HandleFrame {
        client: ClientInfo,
        frame: String,
        res_tx: oneshot::Sender<()>,
    },
}

/// Log frame.
struct Frame {
    cmd: String,
    data: String,
}

impl TryFrom<String> for Frame {
    type Error = String;

    /// `str` should be formatted as `init^cmd^data^endData^close`.
    fn try_from(str: String) -> Result<Self, Self::Error> {
        // Checks if the frame field matches the expected field
        fn check_field(field: &str, expected: &str, err: &str) -> Result<(), String> {
            if field != expected {
                Err(err.format(&[field, expected]))
            } else {
                Ok(())
            }
        }

        let err = "Malformed frame str: found `{}` instead of `{}`";

        let fields: Vec<&str> = str.split('^').collect();

        let [init, cmd, data, end_data, close] = fields[..] else {
            return Err("Malformed frame str: unexpected number of fields".to_string());
        };

        check_field(init, "init", err)?;
        check_field(end_data, "endData", err)?;
        check_field(close, "close", err)?;

        Ok(Frame { cmd: cmd.to_string(), data: data.to_string() })
    }
}

/// A WebSockets server.
///
/// Contains the logic of how connections interact with each other.
///
/// Call and spawn [`run`](Self::run) to start processing commands.
#[derive(Debug)]
pub struct Server {
    /// Map of connection IDs to their message transmiters.
    sessions: HashMap<ConnId, mpsc::UnboundedSender<Msg>>,

    /// Tracks total number of current established connections.
    clients_count: Arc<AtomicUsize>,

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
                clients_count: Arc::new(AtomicUsize::new(0)),
                cmd_rx,
            },
            ServerHandle { cmd_tx },
        )
    }

    /// Get the connection transmiter associated to a client connection ID
    fn get_conn_tx(&self, conn_id: ConnId) -> Result<&mpsc::UnboundedSender<Msg>, ()> {
        if let Some(conn_id) = self.sessions.get(&conn_id) {
            Ok(conn_id)
        } else {
            Err(())
        }
    }

    /// Send message to users.
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
            if let Err(e) = tx.send(msg.clone()) {
                tracing::error!(target: "backend", "Failed to send message to client with uuid: {}. Error: {e}", session.0);
            };
        }
    }

    /// Register new session and assign unique ID to this session.
    async fn connect(&mut self, conn_tx: mpsc::UnboundedSender<Msg>) -> ConnId {
        // Notify all users
        self.send_system_message(None, "Someone joined").await;

        // Register session with random connection ID
        let id = Uuid::new_v4();
        self.sessions.insert(id, conn_tx);

        self.clients_count.fetch_add(1, Ordering::SeqCst);
        let count = self.clients_count.load(Ordering::SeqCst);
        self.send_system_message(None, format!("Connected clients: {count}")).await;

        // Send id back
        id
    }

    /// Attempt to log an incoming "log frame".
    /// Writes an error to stdout if the log frame is malformed.
    async fn handle_frame(&self, client: ClientInfo, frame: String) {
        tracing::debug!(target: "backend", "Client {}:{} sent frame: {frame}", client.ip(), client.port());
        match Frame::try_from(frame) {
            Ok(frame) => {
                const TGT: &'static str = "backend-file";
                let message =
                    format!(
                        "Received frame from {}:{} [ cmd: {}, data: {} ]",
                        client.ip(), client.port(), frame.cmd, frame.data
                    );

                match frame.cmd.to_uppercase().as_str() {
                    "DEBUG" => tracing::debug!(target: TGT, message),
                    "INFO" => tracing::info!(target: TGT, message),
                    "WARN" => tracing::warn!(target: TGT, message),
                    "ERROR" => tracing::error!(target: TGT, message),
                    _ => tracing::trace!(target: TGT, message),
                }
            }
            Err(e) =>
                tracing::error!(target: "backend", e)
        }
    }

    /// Unregister connection from sessions map and broadcast disconnection message.
    async fn disconnect(&mut self, client: ClientInfo, conn_id: ConnId) {
        tracing::info!(target: "backend", "Client {}:{} disconnected", client.ip(), client.port());

        // Remove sender
        if self.sessions.remove(&conn_id).is_none() {
            tracing::error!(target: "backend", "Tried to remove an nonexistent session");
        };
        
        self.clients_count.fetch_sub(1, Ordering::SeqCst);

        // Send message to other users
        self.send_system_message(None, "Someone disconnected").await;
    }

    /// Make the server listen for incoming handler commands.
    pub async fn run(mut self) -> io::Result<()> {
        while let Some(cmd) = self.cmd_rx.recv().await {
            match cmd {
                Command::Connect { conn_tx, res_tx } => {
                    let conn_id = self.connect(conn_tx).await;
                    let _ = res_tx.send(conn_id);
                }

                Command::Disconnect { client, conn } => {
                    self.disconnect(client, conn).await;
                }

                Command::HandleFrame { client, frame, res_tx} => {
                    self.handle_frame(client, frame).await;
                    let _ = res_tx.send(());
                }
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

    /// Unregister message sender and broadcast disconnection message to all users.
    pub fn disconnect(&self, client: ClientInfo, conn: ConnId) {
        // Unwrap: server should not have been dropped
        self.cmd_tx.send(Command::Disconnect { client, conn }).unwrap();
    }
    
    /// Handle an incoming "log frame".
    pub async fn handle_frame(&self, client: ClientInfo, frame: String) {
        let (res_tx, res_rx) = oneshot::channel();

        self.cmd_tx
            .send(Command::HandleFrame { client, frame, res_tx })
            .unwrap();

        res_rx.await.unwrap();
    }
}