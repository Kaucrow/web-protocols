use crate::prelude::*;
use crate::FtpSession;
use anyhow::Result;
use std::fs;
use std::time::UNIX_EPOCH;
use chrono::{ DateTime, Local };

impl FtpSession {
    #[tracing::instrument(
        name = "Listing directory files",
        skip(self, stream)
    )]
    pub async fn list_dir(&mut self, stream: &mut TcpStream) -> Result<()> {
        tracing::debug!("Reading: {:?}", self.real_dir);

        self.ctrl.write_all(b"150 Sending directory list\r\n").await?;

        let mut listing = String::new();

        for entry in fs::read_dir(&self.real_dir)? {
            let entry = entry?;
            let metadata = entry.metadata()?;

            // Get file type (directory or file)
            let is_dir = metadata.is_dir();
            // Get file perms (simplified for Windows)
            let file_perms = if is_dir { "drwxr-xr-x" } else { "-rw-r--r--" };

            let hard_links = 0;
            let uid = 0;
            let gid = 0;

            // Get file size
            let file_size = metadata.len();

            // Get last modified time (formatted as `Month Day Hour:Minute`)
            let modified_time = metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs();
            let datetime = UNIX_EPOCH + std::time::Duration::new(modified_time, 0);

            // Convert SystemTime to DateTime using chrono for proper formatting
            let datetime: DateTime<Local> = DateTime::from(datetime);

            // Format the timestamp
            let timestamp = datetime.format("%b %d %H:%M").to_string();

            // Get the file name
            let file_name = entry.file_name().into_string().unwrap_or_default();

            // Add entry to the listing
            listing.push_str(&format!(
                "{:<10} {:>10} {:>10} {:>10} {:>10} {} {}\r\n",
                file_perms, hard_links, uid, gid, file_size, timestamp, file_name
            ));
        }

        stream.write_all(listing.as_bytes()).await?;

        Ok(self.ctrl.write_all(b"226 Directory send OK\r\n").await?)
    }
}