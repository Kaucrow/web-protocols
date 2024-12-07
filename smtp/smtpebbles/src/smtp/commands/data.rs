use crate::prelude::*;
use crate::SmtpSession;
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

        self.stream.write_all(b"354 Let us see if there is anything important written on this. (Start mail input; end with <CRLF>.<CRLF>)\r\n").await?;
        tracing::debug!(target: "smtp", "DATA command started");

        let mut reader = BufReader::new(&mut self.stream);

        let mut buffer = String::new();

        loop {
            let mut line = String::new();
            let bytes_read = reader.read_line(&mut line).await?;

            line = line.trim_end_matches("\r\n").to_string();

            if bytes_read == 0 || line.trim() == "." {
                break;  // End of message when line contains only a period '.'
            }

            buffer.push_str(&line);
        }

        tracing::info!("Read message data: {:?}", &buffer);

        self.email.as_mut().unwrap().data = Some(buffer);

        self.stream.write_all(b"250 \r\n").await?;

        Ok(())
    }
}
