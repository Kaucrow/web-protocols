use anyhow::Result;
use smtpebbles::{
    telemetry,
    settings::get_settings,
    SmtpServer,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Get the settings
    let settings = get_settings()?;

    // Init the tracing subscriber
    let (subscriber, _guard) = telemetry::get_subscriber().await?;
    telemetry::init_subscriber(subscriber);

    // Build the SMTP server and run it
    let smtp_server = SmtpServer::new(settings)?;
    smtp_server.run().await?;

    Ok(())
}