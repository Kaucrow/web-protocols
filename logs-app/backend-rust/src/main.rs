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
use actix_web::{middleware, web, App, HttpServer};
use backend_rust::settings::get_settings;
use tokio::{
    task::spawn,
    try_join,
};
use backend_rust::ws;
use backend_rust::udp;

mod telemetry;

#[tokio::main]
async fn main() -> io::Result<()> {
    let (subscriber, _guard) = crate::telemetry::get_subscriber();
    crate::telemetry::init_subscriber(subscriber);

    let settings = get_settings().expect("Failed to read settings");

    tracing::info!(target: "backend", "Listening for WebSocket connections on ws://{}:{}/ws", settings.local_ip, settings.ws_port);
    tracing::info!(target: "backend", "Listening for UDP messages on {}:{}", settings.local_ip, settings.udp_port);

    let udp_server_task = spawn(udp::server::run(settings.clone()));

    let (ws_server, ws_server_handle) = ws::Server::new();
    let ws_server_task = spawn(ws_server.run());

    let http_server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(ws_server_handle.clone()))
            // Websocket routes
            .service(web::resource("/ws").route(web::get().to(ws::ws)))
            // Standard middleware
            .wrap(middleware::NormalizePath::trim())
    })
    .bind((settings.host, settings.ws_port))?
    .run();

    try_join!(http_server, async move { ws_server_task.await.unwrap() }, async move { udp_server_task.await.unwrap() })?;

    std::mem::drop(_guard);

    Ok(())
}