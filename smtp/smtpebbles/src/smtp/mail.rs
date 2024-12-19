use crate::{get_postgres_conn_str, prelude::*};
use crate::SmtpSession;
use anyhow::Result;
use std::time::UNIX_EPOCH;
use lettre::{ AsyncTransport, Message };
use lettre::message::{ header::{ ContentType, ContentDisposition }, MultiPart, SinglePart };
use hickory_resolver::{
    TokioAsyncResolver,
    config::*,
};
use mime_guess::mime::TEXT_PLAIN;

impl SmtpSession {
    #[tracing::instrument(
        name = "Sending email",
        skip(self)
    )]
    pub async fn send_email(&self) -> Result<()> {
        let (sender, recipient, recipient_domain, subject, date, content, attachments) = {
            let email = self.email.as_ref().ok_or(anyhow!("Missing email"))?;
            let sender = email.sender.as_ref().ok_or(anyhow!("Missing email sender"))?;
            let recipient = email.recipient.as_ref().ok_or(anyhow!("Missing email recipient"))?;

            let recipient_domain = recipient.splitn(2, '@').nth(1).ok_or(anyhow!("Recipient doesn't contain an `@` delimiter"))?;

            let data = email.data.as_ref().ok_or(anyhow!("Missing email data"))?;
            let subject = data.subject.as_ref().unwrap();
            let date = data.date.as_ref().unwrap();
            let system_time = UNIX_EPOCH + std::time::Duration::from_secs(date.timestamp() as u64)
            + std::time::Duration::from_nanos(date.timestamp_subsec_nanos() as u64);
            let content = data.content.as_ref().unwrap();
            let attachments = &data.attachments;

            (sender, recipient, recipient_domain, subject, system_time, content, attachments)
        };

        let postgres_conn_str = get_postgres_conn_str()?;

        // Create a connection pool
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&postgres_conn_str)
            .await?;

        let email_id: Uuid = query("
            INSERT INTO sent_emails
            (sender_email, destination_email, subject, content, destination_name)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING uuid
        ")
        .bind(sender)
        .bind(recipient)
        .bind(subject)
        .bind(content)
        .bind("")
        .fetch_one(&pool)
        .await?
        .get("uuid");

        let email = if attachments.is_empty() {
            Message::builder()
                .from(sender.parse()?)
                .to(recipient.parse()?)
                .subject(subject)
                .date(date)
                .body(content.to_string())?
        } else {
            let mut multipart = MultiPart::mixed()
                .singlepart(
                    SinglePart::builder()
                        .content_type(ContentType::parse(TEXT_PLAIN.as_ref())?)
                        .body(content.to_owned()),
                );
            
            for attachment in attachments {
                let filename = &attachment.filename;
                let dot_pos = filename.rfind('.');

                let mime_type = if let Some(pos) = dot_pos {
                    let file_ext = &filename[pos + 1..];
                    mime_guess::from_ext(file_ext).first_or(mime_guess::mime::APPLICATION_OCTET_STREAM)
                } else {
                    mime_guess::mime::APPLICATION_OCTET_STREAM
                };

                multipart = multipart.singlepart(
                    SinglePart::builder()
                        .content_type(ContentType::from(mime_type))
                        .header(ContentDisposition::attachment(filename))
                        .body(attachment.body.to_owned())
                );
            }

            Message::builder()
                .from(sender.parse()?)
                .to(recipient.parse()?)
                .subject(subject)
                .date(date)
                .multipart(multipart)?
        };

        let resolver = TokioAsyncResolver::tokio(
            ResolverConfig::default(),
            ResolverOpts::default()
        );

        let mx_records = resolver.mx_lookup(recipient_domain).await?;
        let best_mx = mx_records.iter().min_by_key(|mx| mx.preference());

        if let Some(mx) = best_mx {
            let smtp_server = mx.exchange().to_string();
            tracing::debug!(target: "smtp", "Sending email to SMTP server: {}", smtp_server);
            
            let mailer = lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::
                builder_dangerous(smtp_server).build();

            mailer.send(email).await?;

            Ok(())
        } else {
            Err(anyhow!(format!("No MX records found for {}", recipient_domain)))
        }
    }
}

impl SmtpSession {
    #[tracing::instrument(
        name = "Receiving email",
        skip(self)
    )]
    pub async fn receive_email(&self) -> Result<()> {
        let (sender, recipient, subject, content) = {
            let email = self.email.as_ref().ok_or(anyhow!("Missing email"))?;
            let sender = email.sender.as_ref().ok_or(anyhow!("Missing email sender"))?;
            let recipient = email.recipient.as_ref().ok_or(anyhow!("Missing email recipient"))?;

            let data = email.data.as_ref().ok_or(anyhow!("Missing email data"))?;
            let subject = data.subject.as_ref().unwrap();
            let content = data.content.as_ref().unwrap();

            (sender, recipient, subject, content)
        };

        let postgres_conn_str = get_postgres_conn_str()?;

        // Create a connection pool
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&postgres_conn_str)
            .await?;

        query("
            INSERT INTO received_emails
            (receiver_email, sender_email, subject, content, sender_name)
            VALUES ($1, $2, $3, $4, $5)
        ")
        .bind(recipient)
        .bind(sender)
        .bind(subject)
        .bind(content)
        .bind("")
        .execute(&pool)
        .await?;

        Ok(())
    }
}