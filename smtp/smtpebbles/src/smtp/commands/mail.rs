use crate::prelude::*;
use crate::{ SmtpSession, Email };
use anyhow::Result;

impl SmtpSession {
    #[tracing::instrument(
        name = "Handling MAIL command",
        skip(self)
    )]
    pub async fn mail(&mut self, param: String) -> Result<()> {
        if !param.starts_with("FROM:") {
            self.stream.write_all(b"501 I may have been too rash in bestowing the mark of communication upon you. (Syntax error in parameters or arguments)\r\n").await?;
            return Ok(());
        }

        let sender = param.trim_start_matches("FROM:").trim().trim_matches('<').trim_matches('>');

        if self.email.is_some() {
            self.stream.write_all(b"503 Is this some kind of joke? (Bad sequence of commands)\r\n").await?;
            return Ok(());
        }

        self.email = Some(
            Email {
                sender: Some(sender.to_string()),
                recipient: None,
                data: None,
            }
        );

        self.stream.write_all(b"250 OK: I assume you have come here because you want a way out.\r\n").await?;

        tracing::debug!(target: "smtp", "MAIL FROM accepted: {}", sender);

        Ok(())
    }
}