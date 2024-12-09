use anyhow::Result;
use smtpebbles::{
    prelude::*,
    get_settings,
    telemetry,
    SmtpServer,
    AppServer,
};
use tokio::task;
use lettre::Message;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::AsyncTransport;
use sqlx::{
    postgres::{ PgPoolOptions, PgRow },
    Row,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Get the settings
    let settings = get_settings()?;

    /*
    // SMTP server details for Gmail
    let smtp_server = "smtp.gmail.com";
    let smtp_port = 587;
    let username = "your_email@gmail.com"; // Your Gmail address
    let app_password = "your_app_password"; // Your Gmail App Password

    // Compose the email
    let email = Message::builder()
        .from("testuser@shadedcitadel.xyz".parse::<Mailbox>()?) // Use your custom email
        .to("neoserpent22@gmail.com".parse::<Mailbox>()?) // Recipient's email address
        .subject("Test Email from Rust")
        .body("Hello world".to_string())?;

    // Set up the SMTP transport with credentials
    let creds = Credentials::new("".to_string(), "".to_string());

    let mailer: lettre::AsyncSmtpTransport<lettre::Tokio1Executor> =
        lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::builder_dangerous("shadedcitadel.xyz")
            .build();

    // Send the email
    mailer.send(email).await?;

    println!("Email sent successfully!");
    */

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

    tokio::try_join!(smtp_task, app_task)?;

    Ok(())
}