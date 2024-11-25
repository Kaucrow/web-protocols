use crate::prelude::*;
use crate::FtpSession;
use anyhow::Result;

impl FtpSession {
    #[tracing::instrument(
        name = "Setting up active connection",
        skip(self)
    )]
    pub async fn port(&mut self, addr: &str) -> Result<()> {
        let addr: SocketAddr = {
            let octets: Vec<&str> = addr.split(',').collect();
            if octets.len() == 6 {
                // Extract IP and port from the addr
                let ip = format!("{}.{}.{}.{}", octets[0], octets[1], octets[2], octets[3]);
                let port = (octets[4].parse::<u16>().unwrap() << 8) + octets[5].parse::<u16>().unwrap();

                let socket_addr = format!("{}:{}", ip, port);
                socket_addr.parse()?
            } else {
                bail!("Active data connection address contains an unexpected number of octets")
            }
        };

        let listener = TcpListener::bind(addr).await?;
        self.data = Some(listener);

        Ok(self.ctrl.write_all(b"200 PORT command successful. Consider using PASV\r\n").await?)
    }
}