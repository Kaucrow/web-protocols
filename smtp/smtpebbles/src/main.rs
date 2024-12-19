use anyhow::Result;
use smtpebbles::{
    get_settings,
    telemetry,
    SmtpServer,
    AppServer,
};
use tokio::task;

#[cfg(windows)]
use tokio::signal::windows::ctrl_c;

#[cfg(unix)]
use tokio::signal::unix::{ signal, SignalKind };

#[tokio::main]
async fn main() -> Result<()> {
    // Get the settings
    let settings = get_settings()?;

    // Init the tracing subscriber
    let (subscriber, _guard) = telemetry::get_subscriber().await?;
    telemetry::init_subscriber(subscriber);

    // Build the SMTP server and run it
    let smtp_server = SmtpServer::new(&settings)?;
    let smtp_task = task::spawn(async move {
        if let Err(e) = smtp_server.run().await {
            tracing::error!(target: "smtp", "{:#?}", e);
        }
    });

    // Build the app server and run it
    let app_server = AppServer::build(settings).await?;
    let app_task = task::spawn(async move {
        if let Err(e) = app_server.run_until_stopped().await {
            tracing::error!(target: "app", "{:#?}", e);
        }
    });

    #[cfg(windows)]
    let mut sigint = ctrl_c()?;

    #[cfg(unix)]
    let mut sigint = signal(SignalKind::interrupt())?;

    let signal_task = task::spawn(async move {
        sigint.recv().await.expect("Failed to listen for SIGINT");
        tracing::info!("Received SIGINT, shutting down...");
    });

    tokio::select! {
        _ = app_task => tracing::error!("App server stopped"),
        _ = smtp_task => tracing::error!("SMTP server stopped"),
        _ = signal_task => {},
    }

    Ok(())
}