use crate::{
    prelude::*,
    SmtpSession,
};
use thiserror::Error;
use anyhow::Result;

pub enum CommandResult {
    Continue,
    Exit
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Parameter is missing")]
    MissingParam,
}

impl SmtpSession {
    #[tracing::instrument(
        name = "",
        skip(self, request)
        fields(
            email = ?self.email
        )
    )]
    pub async fn process_command(&mut self, request: &String) -> Result<CommandResult> {
        tracing::debug!("Received request: {}", request);

        let mut req_split = request.splitn(2, ' ');
        let command = req_split.next().ok_or(anyhow!("Command is missing"))?;
        let param = req_split.next();

        match command {
            "HELO" => {
                self.stream.write_all(b"250 A little animal, on the floor of my chamber.\r\n").await?;
            }
            "EHLO" => {
                self.stream.write_all(b"250-A little animal, on the floor of my chamber. I think I know what you are looking for.\r\n250 HELP\r\n").await?;
            }
            "MAIL" => {
                let param = get_param(param, &mut self.stream).await?;
                self.mail(param).await?;
            }
            "RCPT" => {
                let param = get_param(param, &mut self.stream).await?;
                self.rcpt(param).await?;
            }
            "DATA" => {
                self.data().await?;
            }
            "QUIT" => {
                self.stream.write_all(b"221 Best of luck to you, little creature. I must resume my work. (Service closing transmission channel)\r\n").await?;
                return Ok(CommandResult::Exit);
            }
            _ => {
                self.stream.write_all(b"500 Perhaps this is fate. Karma recounting my deeds and bearing its fangs at me in the most ironic fashion. (Syntax error, command unrecognized)\r\n").await?;
            }
        }

        Ok(CommandResult::Continue)
    }
}

async fn get_param<'a>(param: Option<&'a str>, stream: &'a mut TcpStream) -> Result<String> {
    if let Some(param) = param {
        Ok(param.to_string())
    } else {
        stream.write_all(b"501 Syntax error in parameters or arguments\r\n").await?;
        bail!(Error::MissingParam)
    }
}