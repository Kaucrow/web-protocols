use crate::prelude::*;
use crate::FtpSession;
use anyhow::Result;

impl FtpSession {
    pub async fn cwd(&mut self, target_dir: &str) -> Result<()> {
        if target_dir == ".." {
            Ok(self.cdup().await?)
        } else {
            let new_real_dir = if target_dir.starts_with('/') {
                self.server.base_dir.join(target_dir.strip_prefix('/').unwrap())
            } else {
                self.real_dir.join(target_dir)
            };

            if new_real_dir.is_dir() {
                self.real_dir = new_real_dir;
                self.virtual_dir = if target_dir.starts_with('/') {
                    target_dir.to_string()
                } else {
                    format!("{}/{}", self.virtual_dir.trim_end_matches('/'), target_dir)
                };

                Ok(self.ctrl.write_all(b"250 Directory successfully changed.\r\n").await?)
            } else {
                Ok(self.ctrl.write_all(b"550 Failed to change directory.\r\n").await?)
            }
        }
    }

    pub async fn cdup(&mut self) -> Result<()> {
        if self.virtual_dir != "/" {
            self.virtual_dir = Path::new(&self.virtual_dir)
                .parent()
                .unwrap_or(Path::new("/"))
                .to_string_lossy()
                .to_string();

            self.real_dir = self.real_dir.parent().unwrap_or(&self.server.base_dir).to_path_buf();

            Ok(self.ctrl.write_all(b"250 Directory successfully changed\r\n").await?)
        } else {
            Ok(self.ctrl.write_all(b"550 Already at the root directory\r\n").await?)
        }
    }
}