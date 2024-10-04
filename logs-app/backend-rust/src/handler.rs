use std::{
    pin::pin,
    time::{Duration, Instant},
    net::SocketAddr,
};
use actix_web::HttpRequest;
use actix_ws::AggregatedMessage;
use futures_util::{
    future::{select, Either},
    StreamExt as _,
};
use tokio::{sync::mpsc, time::interval};
use crate::{ServerHandle, ConnId};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// Echo text & binary messages received from the client, respond to ping messages, and monitor
/// connection health to detect network issues and free up resources.
pub async fn ws(
    req: HttpRequest,
    server: ServerHandle,
    mut session: actix_ws::Session,
    msg_stream: actix_ws::MessageStream,
) {
    let addr = req.peer_addr().unwrap_or_else(|| SocketAddr::from(([0, 0, 0, 0], 0)));
    let ip = addr.ip();
    let port = addr.port();

    tracing::event!(target: "backend", tracing::Level::INFO, "Client connected from IP {ip} on port {port}.");

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

        // TODO: nested select is pretty gross for readability on the match
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
                        process_text_msg(&server, &text, conn_id)
                            .await;
                    }

                    AggregatedMessage::Binary(_bin) => {
                        tracing::event!(target: "backend", tracing::Level::WARN, "Unexpected binary message");
                    }

                    AggregatedMessage::Close(reason) => break reason,
                }
            }

            // Client WebSocket stream error
            Either::Left((Either::Left((Some(Err(err)), _)), _)) => {
                tracing::event!(target: "backend", tracing::Level::ERROR, "{}", err);
                break None;
            }

            // Client WebSocket stream ended
            Either::Left((Either::Left((None, _)), _)) => break None,

            // Messages received from other room participants
            Either::Left((Either::Right((Some(_msg), _)), _)) => {
                //session.text(msg).await.unwrap();
            }

            // All connection's message senders were dropped
            Either::Left((Either::Right((None, _)), _)) => unreachable!(
                "All connection message senders were dropped; server may have panicked"
            ),

            // Heartbeat internal tick
            Either::Right((_inst, _)) => {
                // If no heartbeat ping/pong received recently, close the connection
                if Instant::now().duration_since(last_heartbeat) > CLIENT_TIMEOUT {
                    tracing::event!(
                        target: "backend",
                        tracing::Level::INFO,
                        "Client has not sent heartbeat in over {CLIENT_TIMEOUT:?}; disconnecting"
                    );
                    break None;
                }

                // Send heartbeat ping
                let _ = session.ping(b"").await;
            }
        };
    };

    server.disconnect(conn_id);

    // attempt to close connection gracefully
    let _ = session.close(close_reason).await;
}

async fn process_text_msg(
    server: &ServerHandle,
    text: &str,
    conn: ConnId,
) {
    // Strip leading and trailing whitespace (spaces, newlines, etc.)
    let msg = text.trim();

    if let Some(idx) = msg.find('^') {
        if &msg[0..idx] == "init" {
            server.handle_frame(conn, msg.to_string()).await;
        }
    }
}