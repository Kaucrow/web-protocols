//! WebSocket server.
//!
//! Open `http://localhost:8080/` in browser to test.

use std::io;

use actix_files::NamedFile;
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

/// Room ID.
pub type RoomId = String;

/// Message sent to a room/client.
pub type Msg = String;

async fn index() -> impl Responder {
    NamedFile::open_async("./static/index.html").await.unwrap()
}

/// Handshake and start WebSocket handler with heartbeats.
async fn ws(
    req: HttpRequest,
    stream: web::Payload,
    server: web::Data<ServerHandle>,
) -> Result<HttpResponse, Error> {
    let (res, session, msg_stream) = actix_ws::handle(&req, stream)?;

    // Spawn websocket handler (and don't await it) so that the response is returned immediately
    spawn_local(handler::ws(
        (**server).clone(),
        session,
        msg_stream,
    ));

    Ok(res)
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    let subscriber = crate::telemetry::get_subscriber();
    crate::telemetry::init_subscriber(subscriber);

    tracing::event!(target: "backend", tracing::Level::INFO, "Listening on http://localhost:8080");

    let (server, server_handle) = Server::new();

    let server = spawn(server.run());

    let http_server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(server_handle.clone()))
            // WebSocket UI HTML file
            .service(web::resource("/").to(index))
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

    Ok(())
}