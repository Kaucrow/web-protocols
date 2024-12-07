use crate::{
    prelude::*,
    smtp::{
        self,
        handler::CommandResult,
    },
    Settings,
};
use anyhow::Result;

pub struct SmtpServer {
    pub host: String,
    pub port: u16,
}

#[derive(Debug)]
pub struct Email {
    pub sender: Option<String>,
    pub recipient: Option<String>,
    pub data: Option<String>,
}

pub struct SmtpSession {
    pub stream: TcpStream,
    pub email: Option<Email>,
}

impl SmtpServer {
    pub fn new(settings: Settings) -> Result<Self> {
        let mut host = settings.smtp.host;

        if host == "0.0.0.0" {
            host = get_local_ip()
                .ok_or(anyhow!("Failed to get the local IPv4 address"))?
                .to_string();
        }

        Ok(Self {
            host,
            port: settings.smtp.port,
        })
    }

    fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub async fn run(self: &Self) -> Result<()> {
        let addr = self.addr();
        let listener = TcpListener::bind(&addr).await?;
        tracing::info!(target: "smtp", "SMTP server listening on {}", &addr);
    
        // Run indefinitely, and for every new client connection, spawn a new task to handle it
        loop {
            let (stream, _) = listener.accept().await?;
            tokio::spawn(async move {
                Self::handle_client(stream).await
            });
        }
    }

    pub async fn handle_client(mut stream: TcpStream) -> Result<()> {
        let client_addr = stream.peer_addr()?;

        // Send the initial greeting
        stream.write_all(b"220 ...is this reaching you?\r\n").await?;

        // Create the session and run it
        let session = SmtpSession::new(stream);
        if let Err(e) = session.run().await {
            if let Some(e) = e.downcast_ref::<io::Error>() {
                if e.kind() == io::ErrorKind::ConnectionReset {
                    tracing::warn!(target: "smtp", "Client disconnected forcibly: {}", client_addr);
                }
            } else {
                tracing::error!(target: "smtp", "Unexpected error in session: {:#?}", e);
            }
        }
        Ok(())
    }
}

impl SmtpSession {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            email: None,
        }
    }

    async fn run(mut self) -> Result<()> {
        // 1KB buffer
        let mut buffer = vec![0; 1024];

        loop {
            let n = self.stream.read(&mut buffer).await?;

            if n == 0 {
                break;
            }

            let request = String::from_utf8_lossy(&buffer[..n]).trim().to_string();

            // Process the command
            match self.process_command(&request).await {
                Ok(CommandResult::Exit) => break,
                Err(e) => {
                    if let Some(e) = e.downcast_ref::<smtp::handler::Error>() {
                        tracing::error!(target: "smtp", "{}", e);
                    } 
                    else {
                        return Err(e)
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}

/// Gets the local IPv4 address.
fn get_local_ip() -> Option<IpAddr> {
    // Create a dummy UDP socket to determine the local IP
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    // Connect to a public address. This doesn't actually send data
    socket.connect("8.8.8.8:80").ok()?;
    // Get the local address used for the connection
    let local_addr = socket.local_addr().ok()?;
    Some(local_addr.ip())
}