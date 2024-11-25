use std::sync::Arc;
use anyhow::Result;
use ftpebbles::{
    settings::{ self, Settings },
    startup::{ FtpServer, Credentials },
    telemetry,
};

#[tokio::main]
async fn main() -> Result<()> {
    let matches = settings::get_matches();

    // If the "help" option was passed, print the help screen and exit
    if matches.contains_id("help") {
        let mut cmd = settings::get_command();
        cmd.print_help().expect("Failed to print help");
        println!();
        return Ok(());
    }

    // Build the settings from the command line arguments
    let settings = Settings::new(&matches)?;

    // Init the tracing subscriber
    let (subscriber, _guard) = telemetry::get_subscriber().await?;
    telemetry::init_subscriber(subscriber);

    // If both the username and password were specified, set the server credentials
    let credentials = settings
        .username
        .zip(settings.password)
        .map(|(username, password)| Credentials { username, password });

    // Build the server and run it
    let server = Arc::new(FtpServer::new(settings.host, settings.port, settings.base_dir, credentials)?);
    server.run().await?;

    Ok(())
}