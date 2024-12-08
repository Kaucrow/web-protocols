use anyhow::Result;
use smtpebbles::{
    telemetry,
    settings::get_settings,
    SmtpServer,
};
use lettre::Message;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::AsyncTransport;

#[tokio::main]
async fn main() -> Result<()> {
    // Get the settings
    let settings = get_settings()?;

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
        lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::relay("shadedcitadel.xyz")
            .unwrap()
            .build();

    // Send the email
    mailer.send(email).await?;

    println!("Email sent successfully!");

    /*
    // Init the tracing subscriber
    let (subscriber, _guard) = telemetry::get_subscriber().await?;
    telemetry::init_subscriber(subscriber);

    // Build the SMTP server and run it
    let smtp_server = SmtpServer::new(settings)?;
    smtp_server.run().await?;
    */
    Ok(())
}