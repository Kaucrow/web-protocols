use crate::prelude::*;
use crate::SmtpSession;
use anyhow::Result;

impl SmtpSession {
    #[tracing::instrument(
        name = "Handling RCPT command",
        skip(self)
    )]
    pub async fn rcpt(&mut self, param: String) -> Result<()> {
        if !param.starts_with("TO:") {
            self.stream.write_all(b"501 I may have been too rash in bestowing the mark of communication upon you. (Syntax error in parameters or arguments)\r\n").await?;
            return Ok(());
        }

        let recipient = param.trim_start_matches("TO:").trim().trim_matches('<').trim_matches('>');

        if let Some(email) = &self.email {
            if email.recipient.is_some() || email.data.is_some() {
                self.stream.write_all(b"503 Is this some kind of joke? (Bad sequence of commands)\r\n").await?;
                return Ok(())
            }
        } else {
            self.stream.write_all(b"503 Is this some kind of joke? (Bad sequence of commands)\r\n").await?;
            return Ok(())
        }

        self.email.as_mut().unwrap().recipient = Some(recipient.to_string());
        
        self.stream.write_all(b"250 OK: However, it seems this delivery was not intended for me.\r\n").await?;
        tracing::debug!(target: "smtp", "RCPT TO accepted: {}", recipient);

        Ok(())
    }
}