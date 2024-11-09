use crate::prelude::*;
use crate::startup::TransferType;
use crate::FtpSession;
use anyhow::Result;

impl FtpSession {
    #[tracing::instrument(
        name = "Processing command",
        skip(self, request)
    )]
    pub async fn process_command(&mut self, request: &String) -> Result<()> {
        tracing::debug!("Received request: {}", request);

        let mut req_split = request.splitn(2, ' ');
        let command = req_split.next().unwrap_or("");
        let param = req_split.next();

        match command {
            "USER" => {
                Ok(self.ctrl.write_all(b"331 Username okay, need password.\r\n").await?)
            }
            "PASS" => {
                Ok(self.ctrl.write_all(b"230 User logged in, proceed.\r\n").await?)
            }
            "SYST" => {
                #[cfg(target_os = "windows")]
                return Ok(self.ctrl.write_all(b"215 Windows_NT\r\n").await?);

                #[cfg(not(target_os = "windows"))]
                return Ok(self.ctrl.write_all(b"215 UNIX Type: L8\r\n").await?);
            }
            "FEAT" => {
                self.ctrl.write_all(b"211 Features:\r\n").await?;
                self.ctrl.write_all(b"UTF8\r\n").await?;
                self.ctrl.write_all(b"211 End\r\n").await?;
                Ok(())
            }
            "PWD" => {
                tracing::debug!("Virtual dir: {}", self.virtual_dir);
                Ok(self.ctrl.write_all(format!("257 \"{}\" is the current directory.\r\n", self.virtual_dir).as_bytes()).await?)
            }
            "TYPE" => {
                let mode = param.ok_or_else(|| anyhow!("Parameter is missing"))?;
                match mode {
                    "A" => {
                        self.transfer_type = TransferType::Ascii;
                        Ok(self.ctrl.write_all(b"200 Type set to A\r\n").await?)
                    }
                    "I" => {
                        self.transfer_type = TransferType::Binary;
                        Ok(self.ctrl.write_all(b"200 Type set to I\r\n").await?)
                    }
                    _ => {
                        Ok(self.ctrl.write_all(b"504 Command not implemented for that parameter\r\n").await?)
                    }
                }
            }
            "PASV" => {
                Ok(self.pasv().await?)
            }
            "LIST" => {
                if let Some(data_listener) = self.data.take() {
                    if let Ok((mut data_stream, _)) = data_listener.accept().await {
                        tracing::info!("Client connected for passive mode transfer");
                        self.list_dir(&mut data_stream).await?;

                        drop(data_stream);
                        tracing::info!("Data connection closed");

                        Ok(())
                    } else {
                        let err = "Failed to accept a connection on the data socket";
                        self.ctrl.write_all(format!("425 {}", err).as_bytes()).await?;
                        bail!(err)
                    }
                } else {
                    let err = "The data socket has not been opened";
                    self.ctrl.write_all(format!("425 {}", err).as_bytes()).await?;
                    bail!(err)
                }
            }
            "CWD" => {
                let target_dir = param.ok_or_else(|| anyhow!("Parameter is missing"))?;
                Ok(self.cwd(target_dir).await?)
            }
            "RETR" => {
                let filename = param.ok_or_else(|| anyhow!("Parameter is missing"))?;
                Ok(self.retr(filename).await?)
            }
            _ => {
                Ok(self.ctrl.write_all(b"502 Command not implemented.\r\n").await?)
            }
        }
    }
}