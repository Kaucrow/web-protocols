use crate::prelude::*;
use crate::FtpSession;
use anyhow::Result;
use rand;
use std::net::IpAddr;

impl FtpSession {
    #[tracing::instrument(
        name = "Setting up passive connection",
        skip(self)
    )]
    pub async fn pasv(&mut self) -> Result<()> {
        let passive_port = 50000 + rand::random::<u8>() as u16 % 1000;

        let listener = TcpListener::bind(format!("{}:{}", self.server.host, passive_port)).await?;
        let local_addr = listener.local_addr()?;

        let ip = local_addr.ip();
        let port = local_addr.port();

        let (response, open_data) = match ip {
            IpAddr::V4(ipv4) => {
                let p1 = (port / 256) as u8;
                let p2 = (port % 256) as u8;
                let octets = ipv4.octets();
                (
                    format!("227 Entering Passive Mode ({},{},{},{},{},{})\r\n",
                        octets[0], octets[1], octets[2], octets[3], p1, p2),
                    true
                )
            }
            IpAddr::V6(_) => {
                (
                    "522 Network protocol not supported (IPv6 is not supported in PASV mode)\r\n".to_string(),
                    false
                )
            }
        };

        self.ctrl.write_all(response.as_bytes()).await?;

        if open_data {
            self.data = Some(listener);
        }

        Ok(())
    }
}