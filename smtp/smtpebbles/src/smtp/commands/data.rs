use crate::prelude::*;
use crate::settings::get_settings;
use crate::{ SmtpSession, EmailData };
use anyhow::Result;

impl SmtpSession {
    #[tracing::instrument(
        name = "Handling DATA command",
        skip(self)
    )]
    pub async fn data(&mut self) -> Result<()> {
        if let Some(email) = &self.email {
            if email.data.is_some() {
                self.stream.write_all(b"503 Is this some kind of joke? (Bad sequence of commands)\r\n").await?;
                return Ok(());
            }
        } else {
            self.stream.write_all(b"503 Is this some kind of joke? (Bad sequence of commands)\r\n").await?;
            return Ok(());
        }

        self.email.as_mut().unwrap().data = Some(EmailData {
            from: None,
            to: None,
            subject: None,
            date: None,
            content: None,
        });

        self.stream.write_all(b"354 Let us see if there is anything important written on this. (Start mail input; end with <CRLF>.<CRLF>)\r\n").await?;
        tracing::debug!(target: "smtp", "DATA command started");

        let mut reader = BufReader::new(&mut self.stream);

        let mut content_buf = String::new();
        let mut set_headers = false;

        loop {
            let mut line = String::new();
            let bytes_read = reader.read_line(&mut line).await?;

            line = line.trim_end_matches("\r\n").to_string();

            if bytes_read == 0 || line.trim() == "." {
                break;  // End of message when line contains only a period '.'
            }

            if !set_headers {
                match line {
                    _ if line.starts_with("From: ") => {
                        let from = line.splitn(2, ": ").nth(1).unwrap().trim();
                        self.email.as_mut().unwrap().data.as_mut().unwrap().from = Some(from.to_string());
                    }
                    _ if line.starts_with("To: ") => {
                        let to = line.splitn(2, ": ").nth(1).unwrap().trim();
                        self.email.as_mut().unwrap().data.as_mut().unwrap().to = Some(to.to_string());
                    }
                    _ if line.starts_with("Subject: ") => {
                        let subject = line.splitn(2, ": ").nth(1).unwrap().trim();
                        self.email.as_mut().unwrap().data.as_mut().unwrap().subject = Some(subject.to_string());
                    }
                    _ if line.starts_with("Date: ") => {
                        let date_str = line.splitn(2, ": ").nth(1).unwrap().trim();
                        let date = DateTime::parse_from_rfc2822(date_str)?.with_timezone(&Utc);
                        self.email.as_mut().unwrap().data.as_mut().unwrap().date = Some(date);
                    }
                    _ if line.trim().is_empty() => {
                        set_headers = true;
                    }
                    _ => {}
                }
            } else {
                content_buf.push_str(&line);
            }
        }

        tracing::debug!(target: "smtp", "Read message data: {:?}", &content_buf);

        self.email.as_mut().unwrap().data.as_mut().unwrap().content = Some(content_buf);

        let data = self.email.as_mut().unwrap().data.as_mut().unwrap();

        tracing::debug!(target: "smtp", "DATA: {:#?}", data);

        data.to.get_or_insert("<>".to_string());
        data.from.get_or_insert("undisclosed-recipients".to_string());
        data.subject.get_or_insert("".to_string());
        data.date.get_or_insert(Utc::now());

        self.stream.write_all(b"250 OK: Not that it solves anyone's problem but yours.\r\n").await?;

        let (sender_domain, recipient_domain) = {
            let email = self.email.as_ref().unwrap();

            let sender = email.sender.as_ref().unwrap();
            let recipient = email.recipient.as_ref().unwrap();

            (
                sender.splitn(2, '@').nth(1).ok_or(anyhow!("Malformed sender address"))?,
                recipient.splitn(2, '@').nth(1).ok_or(anyhow!("Malformed sender address"))?
            )
        };

        let settings = get_settings()?;

        match settings.domain {
            domain if domain == sender_domain => self.send_email().await?,
            domain if domain == recipient_domain => self.receive_email().await?,
            _ => {}
        }

        Ok(())
    }
}