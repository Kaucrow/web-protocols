pub mod prelude;
pub mod settings;
pub mod telemetry;
pub mod smtp;

pub use settings::Settings;
pub use smtp::startup::{ SmtpServer, SmtpSession, Email, EmailData };