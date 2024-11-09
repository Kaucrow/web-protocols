use anyhow::Result;
use server_rust::startup::FtpServer;

mod telemetry;

#[tokio::main]
async fn main() -> Result<()> {
    let (subscriber, _guard) = telemetry::get_subscriber();
    telemetry::init_subscriber(subscriber);

    let server = FtpServer::new("127.0.0.1", 21, "C:/Users")?;
    server.run().await?;

    Ok(())
}