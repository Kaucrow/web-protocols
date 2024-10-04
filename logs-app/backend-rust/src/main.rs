//! WebSocket server which parses a "log frame" formatted as `init^cmd^data^endData^close`, and logs its contents to a file.
//! 
//! ## Frame Format
//! - `init`: Frame start identifier.
//! - `cmd`: The log command, representing the log type. Possible values:
//!     - `info`
//!     - `debug`
//!     - `warn`
//!     - `error`
//! 
//!     Any other value gets logged as `trace`.
//! - `data`: The log message or information to be recorded.
//! - `endData`: Frame end indentifier for the data section.
//! - `close`: Frame closure identifier.
//! 
//! ## Example
//! A log frame could look like this:
//! 
//! ```text
//! init^info^Application started successfully^endData^close
//! ```
//!
//! This WebSocket server listens for such frames, parses the log data, and records the appropriate log entries based on the provided `cmd`.
//! 
//! ## Usage
//! Connect via WebSocket (e.g., with Postman) to `ws://localhost:8080/ws` to test.

#![warn(unused_extern_crates)]

use std::io;
use actix_web::{middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use tokio::{
    task::{spawn, spawn_local},
    try_join,
};
use uuid::Uuid;

mod handler;
mod server;
mod telemetry;

pub use self::server::{Server, ServerHandle};

/// Connection ID.
pub type ConnId = Uuid;

/// Message sent to a client.
pub type Msg = String;

/// Handshake and start WebSocket handler with heartbeats.
async fn ws(
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

#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    let (subscriber, _guard) = crate::telemetry::get_subscriber();
    crate::telemetry::init_subscriber(subscriber);

    tracing::event!(target: "backend", tracing::Level::INFO, "Listening on http://localhost:8080");

    let (server, server_handle) = Server::new();

    let server = spawn(server.run());

    let http_server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(server_handle.clone()))
            // Websocket routes
            .service(web::resource("/ws").route(web::get().to(ws)))
            // Standard middleware
            .wrap(middleware::NormalizePath::trim())
            .wrap(actix_web::middleware::Logger::default())
    })
    .workers(2)
    .bind(("127.0.0.1", 8080))?
    .run();

    try_join!(http_server, async move { server.await.unwrap() })?;

    std::mem::drop(_guard);

    Ok(())
}