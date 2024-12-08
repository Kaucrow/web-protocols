use crate::prelude::*;
use crate::SmtpSession;
use anyhow::Result;
use hickory_resolver::TokioAsyncResolver;
use hickory_resolver::config::*;
use lettre::{ SmtpTransport, Message, Transport };
use lettre::transport::smtp::{ SmtpTransportBuilder };
use lettre::message::Mailbox;
use tokio_rustls::rustls::crypto::CryptoProvider;
/*use lettre::{SmtpClient, Transport, Message};
use lettre::smtp::authentication::Credentials;
use lettre::smtp::SmtpTransport;
use lettre::transport::smtp::SmtpResult;*/

impl SmtpSession {
    #[tracing::instrument(
        name = "Sending email",
        skip(self)
    )]
    pub async fn send_email(&self) -> Result<()> {
        let email = self.email.as_ref().ok_or(anyhow!("Missing email"))?;
        let from_addr = email.sender.as_ref().ok_or(anyhow!("Missing email sender"))?;
        let to_addr = &email.recipient.as_ref().ok_or(anyhow!("Missing email recipient"))?;

        let data = email.data.as_ref().ok_or(anyhow!("Missing email data"))?;
        let subject = data.subject.as_ref().unwrap();
        let date = data.date.as_ref().unwrap();
        let content = data.subject.as_ref().unwrap();

        let recipient_domain = to_addr.split('@').nth(1).unwrap();

        let resolver = TokioAsyncResolver::tokio(
            ResolverConfig::default(),
            ResolverOpts::default()
        );
        let mx_records = resolver.mx_lookup(recipient_domain).await?;

        let best_mx = mx_records.iter().min_by_key(|mx| mx.preference());

        if let Some(mx) = best_mx {
            let smtp_server = mx.exchange().to_string();
            tracing::debug!("Using SMTP server: {}", smtp_server);
            let email = Message::builder()
                .from(from_addr.parse()?)
                .to(to_addr.parse()?)
                .subject(subject)
                .body(content.clone())?;

            let mailer = SmtpTransport::relay(&smtp_server)?
                .port(587)
                .build();
            
            mailer.send(&email)?;

        } else {
            tracing::debug!("No MX records found for {}", recipient_domain);
        }

        Ok(())
    }
}