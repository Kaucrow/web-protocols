//! WebSockets server definitions.

pub mod handler;
pub mod server;

pub use server::Server;

use actix_web::{web, Error, HttpRequest, HttpResponse};
use tokio::task::spawn_local;
use crate::ws::server::ServerHandle;

/// Handshake and start WebSocket handler with heartbeats.
pub async fn ws(
    req: HttpRequest,
    stream: web::Payload,
    server: web::Data<ServerHandle>,
) -> Result<HttpResponse, Error> {
    let (res, session, msg_stream) = actix_ws::handle(&req, stream)?;

    // Spawn websocket handler (and don't await it) so that the response is returned immediately
    spawn_local(handler::ws(
        req,
        (**server).clone(),
        session,
        msg_stream,
    ));

    Ok(res)
}