use anyhow::Result;
use server_rust::{
    settings::{ self, Settings },
    startup::FtpServer,
};

mod telemetry;

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

    let (subscriber, _guard) = telemetry::get_subscriber();
    telemetry::init_subscriber(subscriber);

    let server = FtpServer::new(settings.host, settings.port, settings.base_dir)?;
    server.run().await?;

    Ok(())
}