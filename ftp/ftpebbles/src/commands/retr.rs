use crate::prelude::*;
use crate::{ FtpSession, TransferOptions, TransferType };
use super::convert_to_ascii;
use anyhow::Result;

impl FtpSession {
    #[tracing::instrument(
        name = "Performing file retrieval",
        skip(self, filename)
    )]
    pub async fn retr(&mut self, filename: &str) -> Result<()> {
        let file_path = self.real_dir.join(filename);
        tracing::debug!("File: {:?}", file_path);

        let mut file = match File::open(&file_path).await {
            Ok(mut file) => {
                let opts = self.transfer_opts.take().unwrap_or(TransferOptions::default());

                if let Some(offset) = opts.offset {
                    file.seek(tokio::io::SeekFrom::Start(offset)).await?;
                    file
                } else {
                    file
                }
            },
            Err(e) => {
                self.ctrl.write_all(b"550 File not found\r\n").await?;
                bail!(e);
            }
        };

        self.ctrl.write_all(b"150 Opening data connection\r\n").await?;

        if let Some(data_listener) = self.data.take() {
            if let Ok((mut data_stream, _)) = data_listener.accept().await {
                // 64KB buffer
                let mut buffer = [0; 65536];
                loop {
                    let bytes_read = file.read(&mut buffer).await?;
                    if bytes_read == 0 {
                        break;
                    }

                    if self.transfer_type == TransferType::Ascii {
                        // Convert line endings if needed (e.g., `\n` -> `\r\n`)
                        let ascii_data = convert_to_ascii(&buffer[..bytes_read]);
                        data_stream.write_all(&ascii_data).await?;
                    } else {
                        // Binary transfer, send data as-is
                        data_stream.write_all(&buffer[..bytes_read]).await?;
                    }
                }
                drop(data_stream);
                Ok(self.ctrl.write_all(b"226 Transfer complete\r\n").await?)
            } else {
                Ok(self.ctrl.write_all(b"425 Failed to open data connection\r\n").await?)
            }
        } else {
            Ok(self.ctrl.write_all(b"425 Use PASV first\r\n").await?)
        }
    }
}