use crate::prelude::*;
use crate::FtpSession;
use anyhow::Result;
use std::fs;

impl FtpSession {
    #[tracing::instrument(
        name = "Deleting file"
        skip(self, path)
    )]
    pub async fn delete(&mut self, path: &str) -> Result<()> {
        let sanitized_path = path.trim_start_matches(['/', '\\']);
        let path = self.server.base_dir.join(sanitized_path);

        tracing::debug!("Deleting: {:?}", path);

        if path.exists() && path.is_file() {
            match fs::remove_file(&path) {
                Ok(_) => self.ctrl.write_all(b"250 File deleted successfully\r\n").await?,
                Err(e) => self.ctrl.write_all(format!("550 Could not delete file: {}\r\n", e).as_bytes()).await?,
            }
        } else {
            self.ctrl.write_all(b"550 File not found").await?
        }

        Ok(())
    }
}