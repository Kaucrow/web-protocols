use crate::prelude::*;
use anyhow::Result;
use std::net::{ IpAddr, UdpSocket };

#[derive(Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Clone)]
pub struct FtpServer {
    pub host: String,
    pub port: u16,
    pub base_dir: PathBuf,
    pub credentials: Option<Credentials>,
}

#[derive(PartialEq)]
pub enum TransferType {
    Ascii,
    Binary,
}

#[derive(Clone, Copy)]
pub struct TransferOptions {
    pub offset: Option<u64>,
    pub append: bool,
}

impl Default for TransferOptions {
    fn default() -> Self {
        Self {
            offset: None,
            append: false,
        }
    }
}

pub struct FtpSession {
    pub server: FtpServer,

    pub username: Option<String>,

    pub real_dir: PathBuf,
    pub virtual_dir: String,

    pub ctrl: TcpStream,
    pub data: Option<TcpListener>,

    pub transfer_type: TransferType,

    pub transfer_opts: Option<TransferOptions>,
}

impl FtpServer {
    pub fn new(mut host: String, port: u16, base_dir: String, credentials: Option<Credentials>) -> Result<Self> {
        let base_dir = Path::new(&base_dir).to_path_buf();
        if !base_dir.is_dir() {
            bail!("{:?} is not a directory", base_dir);
        }

        // If the host is 0.0.0.0, replace use the local IPv4 address
        if host == "0.0.0.0" {
            host = get_local_ip()
                .ok_or(anyhow!("Failed to get the local IPv4 address"))?
                .to_string();
        }

        Ok(Self {
            host,
            port,
            base_dir,
            credentials,
        })
    }

    fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub async fn run(&self) -> Result<()> {
        let addr = self.addr();
        let listener = TcpListener::bind(&addr).await?;
        tracing::info!("FTP server serving {:?} on {}", self.base_dir, &addr);

        loop {
            let (stream, _) = listener.accept().await?;
            tokio::spawn(Self::handle_client(self.clone(), stream, self.base_dir.clone()));
        }
    }

    pub async fn handle_client(server: FtpServer, stream: TcpStream, base_dir: PathBuf) -> Result<()> {
        let session = FtpSession::new(server, stream, base_dir);
        if let Err(e) = session.run().await {
            tracing::error!(target: "server", "Error in session: {:#?}", e);
        }
        Ok(())
    }
}

impl FtpSession {
    fn new(server: FtpServer, stream: TcpStream, base_dir: PathBuf) -> Self {
        Self {
            server,
            username: None,
            real_dir: base_dir,
            virtual_dir: String::from("/"),
            ctrl: stream,
            data: None,
            transfer_type: TransferType::Ascii,
            transfer_opts: None,
        }
    }

    // Run function to handle the session lifecycle
    async fn run(mut self) -> Result<()> {
        // Send a welcome message
        self.send_response("220 Welcome to the Five Tiny Pebbles FTP Server\r\n").await?;

        let mut buffer = vec![0; 1024];

        loop {
            let n = self.ctrl.read(&mut buffer).await?;

            if n == 0 {
                break; // Connection closed by client
            }

            let request = String::from_utf8_lossy(&buffer[..n]).trim().to_string();

            // Process the command
            self.process_command(&request).await?;
        }

        Ok(())
    }

    async fn send_response(&mut self, response: &str) -> tokio::io::Result<()> {
        self.ctrl.write_all(response.as_bytes()).await
    }
}

fn get_local_ip() -> Option<IpAddr> {
    // Create a dummy UDP socket to determine the local IP
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    // Connect to a public address. This doesn't actually send data
    socket.connect("8.8.8.8:80").ok()?;
    // Get the local address used for the connection
    let local_addr = socket.local_addr().ok()?;
    Some(local_addr.ip())
}