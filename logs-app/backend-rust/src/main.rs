use actix_web::{middleware::Logger, web, App, HttpRequest, HttpServer, Responder};
use actix_ws::Message;
use futures_util::StreamExt;

async fn ws(req: HttpRequest, body: web::Payload) -> actix_web::Result<impl Responder> {
    println!("Accessing /ws");
    
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Ping(bytes) => {
                    if session.pong(&bytes).await.is_err() {
                        return;
                    }
                }
                Message::Text(mgs) => println!("Got text: {mgs}"),
                _ => break,
            }
        }

        let _ = session.close(None).await;
    });

    Ok(response)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .route("/ws", web::get().to(ws))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;

    Ok(())
}