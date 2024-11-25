use anyhow::Result;
use ftpebbles::{
    settings::{ self, Settings },
    startup::{ FtpServer, Credentials },
    telemetry,
};

#[tokio::main]
async fn main() -> Result<()> {
    let matches = settings::get_matches();

    if matches.contains_id("help") {
        let mut cmd = settings::get_command();
        cmd.print_help().expect("Failed to print help");
        println!();
        return Ok(());
    }

    let settings = Settings::new(&matches)?;

    let (subscriber, _guard) = telemetry::get_subscriber().await?;
    telemetry::init_subscriber(subscriber);

    let credentials = settings
        .username
        .zip(settings.password)
        .map(|(username, password)| Credentials { username, password });

    let server = FtpServer::new(settings.host, settings.port, settings.base_dir, credentials)?;
    server.run().await?;

    Ok(())
}