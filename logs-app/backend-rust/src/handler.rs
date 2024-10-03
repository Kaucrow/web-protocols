use std::{
    pin::pin,
    time::{Duration, Instant},
};
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
    server: ServerHandle,
    mut session: actix_ws::Session,
    msg_stream: actix_ws::MessageStream,
) {
    log::info!("Connected");

    let mut name = None;
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
            // commands & messages received from client
            Either::Left((Either::Left((Some(Ok(msg)), _)), _)) => {
                log::debug!("msg: {msg:?}");

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
                        process_text_msg(&server, &mut session, &text, conn_id, &mut name)
                            .await;
                    }

                    AggregatedMessage::Binary(_bin) => {
                        log::warn!("unexpected binary message");
                    }

                    AggregatedMessage::Close(reason) => break reason,
                }
            }

            // Client WebSocket stream error
            Either::Left((Either::Left((Some(Err(err)), _)), _)) => {
                log::error!("{}", err);
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
                    log::info!(
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
    session: &mut actix_ws::Session,
    text: &str,
    conn: ConnId,
    name: &mut Option<String>,
) {
    // Strip leading and trailing whitespace (spaces, newlines, etc.)
    let msg = text.trim();

    // prefix message with our name, if assigned
    let msg = match name {
        Some(ref name) => format!("{name}: {msg}"),
        None => msg.to_owned(),
    };

    //server.send_message(conn, msg).await
}