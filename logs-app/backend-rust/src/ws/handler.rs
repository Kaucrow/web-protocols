//! Server connection handler.

use std::{
    pin::pin,
    time::{Duration, Instant},
};
use actix_web::HttpRequest;
use actix_ws::AggregatedMessage;
use futures_util::{
    future::{select, Either},
    StreamExt as _,
};
use tokio::{sync::mpsc, time::interval};
use crate::{
    ClientInfo,
    has_init,
    ws::server::ServerHandle,
};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// Handle text & binary messages received from the client, respond to ping messages,
/// and monitor connection health to detect network issues and free up resources.
pub async fn ws(
    req: HttpRequest,
    server: ServerHandle,
    mut session: actix_ws::Session,
    msg_stream: actix_ws::MessageStream,
) {
    let client = ClientInfo::from(&req);

    tracing::info!(target: "backend", "Client {}:{} connected", client.ip(), client.port());

    let mut last_heartbeat = Instant::now();
    let mut interval = interval(HEARTBEAT_INTERVAL);

    // Client websocket tx and rx
    let (conn_tx, mut conn_rx) = mpsc::unbounded_channel();

    // Unwrap: server is not dropped before the HTTP server
    let conn_id = server.connect(conn_tx).await;

    let msg_stream = msg_stream
        .max_frame_size(128 * 1024)
        .aggregate_continuations()
        .max_continuation_size(2 * 1024 * 1024);

    // Messages from the websocket connection
    let mut msg_stream = pin!(msg_stream);

    let close_reason = loop {
        // Most of the futures we process need to be stack-pinned to work with select()
        let tick = pin!(interval.tick());

        // Messages from other connected clients
        let msg_rx = pin!(conn_rx.recv());

        let messages = pin!(select(msg_stream.next(), msg_rx));

        match select(messages, tick).await {
            // Commands & messages received from client
            Either::Left((Either::Left((Some(Ok(msg)), _)), _)) => {
                match msg {
                    AggregatedMessage::Ping(bytes) => {
                        last_heartbeat = Instant::now();
                        // unwrap:
                        session.pong(&bytes).await.unwrap();
                    }

                    AggregatedMessage::Pong(_) => {
                        last_heartbeat = Instant::now();
                    }

                    AggregatedMessage::Text(text) => {
                        process_ws_text_msg(client.clone(), &server, &text)
                            .await;
                    }

                    AggregatedMessage::Binary(_bin) => {
                        tracing::warn!(target: "backend", "Unexpected binary message");
                    }

                    AggregatedMessage::Close(reason) => break reason,
                }
            }

            // Client WebSocket stream error
            Either::Left((Either::Left((Some(Err(err)), _)), _)) => {
                tracing::error!(target: "backend", "{}", err);
                break None;
            }

            // Client WebSocket stream ended
            Either::Left((Either::Left((None, _)), _)) => break None,

            // Messages received from the server or other connected clients
            Either::Left((Either::Right((Some(msg), _)), _)) => {
                if let Err(_) = session.text(msg).await {
                    tracing::debug!(target: "backend", "Text to {}:{} failed. Disconnecting...", client.ip(), client.port());
                    break None;
                }
            }

            // All connections' message senders were dropped
            Either::Left((Either::Right((None, _)), _)) => unreachable!(
                "All connection message senders were dropped; server may have panicked"
            ),

            // Heartbeat internal tick
            Either::Right((_inst, _)) => {
                // If no heartbeat ping/pong received recently, close the connection
                if Instant::now().duration_since(last_heartbeat) > CLIENT_TIMEOUT {
                    tracing::info!(
                        target: "backend",
                        "Client has not sent heartbeat in over {CLIENT_TIMEOUT:?}; disconnecting"
                    );
                    break None;
                }

                // Send heartbeat ping
                let _ = session.ping(b"").await;
            }
        };
    };

    server.disconnect(client, conn_id);

    // Attempt to close connection gracefully
    let _ = session.close(close_reason).await;
}

pub async fn process_ws_text_msg(
    client: ClientInfo,
    server: &ServerHandle,
    text: &str,
) {
    // Strip leading and trailing whitespace (spaces, newlines, etc.)
    let msg = text.trim();

    // Frame message
    if has_init(msg) {   
        server.handle_frame(client,  msg.to_string()).await;
    }
}