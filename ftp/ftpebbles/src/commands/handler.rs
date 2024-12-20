use crate::prelude::*;
use crate::startup::{ TransferOptions, TransferType };
use crate::FtpSession;
use anyhow::Result;

static PARAM_MISSING_ERR: &'static str = "Parameter is missing";

impl FtpSession {
    #[tracing::instrument(
        name = "",
        skip(self, request)
        fields(
            username = %self.username.as_deref().unwrap_or("Undefined")
        )
    )]
    pub async fn process_command(&mut self, request: &String) -> Result<()> {
        tracing::debug!("Received request: {}", request);

        let mut req_split = request.splitn(2, ' ');
        let command = req_split.next().ok_or(anyhow!("Command is missing"))?;
        let param = req_split.next();

        match command {
            "USER" => {
                let user = param.ok_or(anyhow!(PARAM_MISSING_ERR))?;
                self.username = Some(user.to_string());
                Ok(self.ctrl.write_all(b"331 Username okay, need password.\r\n").await?)
            }
            "PASS" => {
                let password = param.ok_or(anyhow!(PARAM_MISSING_ERR))?;

                if let Some(credentials) = &self.server.credentials {
                    if let Some(username) = &self.username {
                        if *username == credentials.username && password == credentials.password {
                            Ok(self.ctrl.write_all(b"230 User logged in, proceed.\r\n").await?)
                        } else {
                            self.username = None;
                            Ok(self.ctrl.write_all(b"530 Not logged in. Authentication failed.\r\n").await?)
                        }
                    } else {
                        Ok(self.ctrl.write_all(b"530 Not logged in. Send USER before PASS.\r\n").await?)
                    }
                } else {
                    Ok(self.ctrl.write_all(b"230 User logged in, proceed.\r\n").await?)
                }
            }
            "SYST" => {
                #[cfg(target_os = "windows")]
                return Ok(self.ctrl.write_all(b"215 Windows_NT\r\n").await?);

                #[cfg(not(target_os = "windows"))]
                return Ok(self.ctrl.write_all(b"215 UNIX Type: L8\r\n").await?);
            }
            "FEAT" => {
                self.ctrl.write_all(b"211 Features:\r\nUTF8\r\n211 End\r\n").await?;
                Ok(())
            }
            "PWD" => {
                tracing::debug!("Virtual dir: {}", self.virtual_dir);
                Ok(self.ctrl.write_all(format!("257 \"{}\" is the current directory.\r\n", self.virtual_dir).as_bytes()).await?)
            }
            "TYPE" => {
                let mode = param.ok_or(anyhow!(PARAM_MISSING_ERR))?;
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
            "PORT" => {
                let addr = param.ok_or(anyhow!(PARAM_MISSING_ERR))?;
                Ok(self.port(addr).await?)
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
                        self.ctrl.write_all(format!("425 {}\r\n", err).as_bytes()).await?;
                        bail!(err)
                    }
                } else {
                    let err = "The data socket has not been opened";
                    self.ctrl.write_all(format!("425 {}\r\n", err).as_bytes()).await?;
                    bail!(err)
                }
            }
            "CWD" => {
                let target_dir = param.ok_or(anyhow!(PARAM_MISSING_ERR))?;
                Ok(self.cwd(target_dir).await?)
            }
            "CDUP" => {
                Ok(self.cdup().await?)
            }
            "RETR" => {
                let filename = param.ok_or(anyhow!(PARAM_MISSING_ERR))?;
                Ok(self.retr(filename).await?)
            }
            "STOR" => {
                let filename = param.ok_or(anyhow!(PARAM_MISSING_ERR))?;
                Ok(self.stor(filename).await?)
            }
            "REST" => {
                let offset = param.ok_or(anyhow!(PARAM_MISSING_ERR))?;
                self.transfer_opts = Some(TransferOptions {
                    offset: Some(offset.parse::<u64>()?),
                    append: false,
                });
                Ok(self.ctrl.write_all(b"350 Restarting at specified offset\r\n").await?)
            }
            "APPE" => {
                self.transfer_opts = Some(TransferOptions {
                    offset: None,
                    append: true,
                });
                Ok(self.ctrl.write_all(b"350 Restarting at specified offset\r\n").await?)
            }
            "DELE" => {
                let path = param.ok_or(anyhow!(PARAM_MISSING_ERR))?;
                Ok(self.delete(path).await?)
            }
            "OPTS" => {
                let param = param.ok_or(anyhow!(PARAM_MISSING_ERR))?;
                match param {
                    "REST STOR" => Ok(self.ctrl.write_all(b"200 Resuming file uploads supported\r\n").await?),
                    _ => Ok(self.ctrl.write_all(b"502 Command not implemented.\r\n").await?)
                }
            }
            _ => {
                Ok(self.ctrl.write_all(b"502 Command not implemented.\r\n").await?)
            }
        }
    }
}