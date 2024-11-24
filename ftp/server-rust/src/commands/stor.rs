use crate::prelude::*;
use crate::{ FtpSession, TransferType, TransferOptions };
use super::convert_to_ascii;
use anyhow::Result;

impl FtpSession {
    #[tracing::instrument(
        name = "Performing file storage",
        skip(self, filename)
    )]
    pub async fn stor(&mut self, filename: &str) -> Result<()> {
        let file_path = self.real_dir.join(filename);
        tracing::debug!("File: {:?}", file_path);

        let opts = self.transfer_opts.unwrap_or(TransferOptions::default());

        let mut file = if let Some(offset) = opts.offset {
            // If REST offset is set, open the file and seek the offset
            let mut file =
                OpenOptions::new()
                    .write(true)
                    .open(file_path)
                    .await?;

            file.seek(tokio::io::SeekFrom::Start(offset)).await?;
            file
        } else if opts.append {
            // If APPE was used, open the file for appending
            OpenOptions::new()
                .write(true)
                .append(true)
                .open(file_path)
                .await?
        } else {
            // If neither REST nor APPE were used, the file should be created
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(file_path)
                .await?
        };

        self.ctrl.write_all(b"150 File status okay; about to open data connection\r\n").await?;

        if let Some(data_listener) = self.data.take() {
            if let Ok((mut data_stream, _)) = data_listener.accept().await {
                tracing::debug!("HERE");
                // 64KB buffer
                let mut buffer = [0; 65536];
                loop {
                    let bytes_read = data_stream.read(&mut buffer).await?;
                    tracing::debug!("Bytes read: {}", bytes_read);
                    if bytes_read == 0 {
                        break;
                    }

                    if self.transfer_type == TransferType::Ascii {
                        // Convert line endings if needed (e.g., `\n` -> `\r\n`)
                        let ascii_data = convert_to_ascii(&buffer[..bytes_read]);
                        file.write(&ascii_data).await?;
                        tracing::debug!("Wrote {} bytes (ASCII) to file", ascii_data.len());
                    } else {
                        // Binary transfer, read data as-is
                        file.write(&buffer[..bytes_read]).await?;
                        tracing::debug!("Wrote {} bytes (binary) to file", bytes_read);
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