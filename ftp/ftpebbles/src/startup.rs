use crate::prelude::*;
use anyhow::Result;
use std::net::{ IpAddr, UdpSocket };
use std::sync::Arc;

/// Auth credentials that the server expects.
#[derive(Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

/// FTP Server struct.
#[derive(Clone)]
pub struct FtpServer {
    pub host: String,
    pub port: u16,
    pub base_dir: PathBuf,  // The directory to serve the files from
    pub credentials: Option<Credentials>,
}

/// File transfer type.
#[derive(PartialEq)]
pub enum TransferType {
    Ascii,
    Binary,
}

/// File transfer options.
/// Specifies whether the transfer should append to the file
/// or start at a specific offset.
#[derive(Clone, Copy)]
pub struct TransferOptions {
    pub offset: Option<u64>,    // Starting byte for the transfer, if specified
    pub append: bool,           // Indicates if the transfer should append to the file 
}

impl Default for TransferOptions {
    fn default() -> Self {
        Self {
            offset: None,
            append: false,
        }
    }
}

/// FTP Session struct.
/// A new session is built for every client that
/// connects to the server.
pub struct FtpSession {
    pub server: Arc<FtpServer>,  // Server data

    pub username: Option<String>,   // Username that the client logged in with or is trying to log in with

    pub real_dir: PathBuf,          // The actual file system path on the server corresponding to the current directory
    pub virtual_dir: String,        // The virtual directory path presented to the client (does not expose the real server path)

    pub ctrl: TcpStream,            // Control channel
    pub data: Option<TcpListener>,  // Data channel. Is Some when open, and None when closed

    pub transfer_type: TransferType,    // Transfer type currently set

    pub transfer_opts: Option<TransferOptions>,    // File transfer options
}

impl FtpServer {
    pub fn new(mut host: String, port: u16, base_dir: String, credentials: Option<Credentials>) -> Result<Self> {
        let base_dir = Path::new(&base_dir).to_path_buf();
        if !base_dir.is_dir() {
            bail!("{:?} is not a directory", base_dir);
        }

        // If the host is 0.0.0.0, use the local IPv4 address as the host address
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

    pub async fn run(self: Arc<Self>) -> Result<()> {
        let addr = self.addr();
        let listener = TcpListener::bind(&addr).await?;
        tracing::info!("FTP server serving {:?} on {}", self.base_dir, &addr);

        // Run indefinitely, and for every new client connection, spawn a new task to handle it
        loop {
            let (stream, _) = listener.accept().await?;
            let server = Arc::clone(&self);
            tokio::spawn(async move {
                Self::handle_client(server, stream).await
            });
        }
    }

    pub async fn handle_client(server: Arc<FtpServer>, stream: TcpStream) -> Result<()> {
        let session = FtpSession::new(server, stream);
        if let Err(e) = session.run().await {
            tracing::error!("Error in session: {:#?}", e);
        }
        Ok(())
    }
}

impl FtpSession {
    fn new(server: Arc<FtpServer>, stream: TcpStream) -> Self {
        Self {
            real_dir: server.base_dir.clone(),
            server,
            username: None,
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
        self.ctrl.write_all(b"220 Welcome to the Five Tiny Pebbles FTP Server\r\n").await?;

        // 1KB buffer
        let mut buffer = vec![0; 1024];

        loop {
            let n = self.ctrl.read(&mut buffer).await?;

            if n == 0 {
                break;  // Connection closed by client
            }

            let request = String::from_utf8_lossy(&buffer[..n]).trim().to_string();

            // Process the command
            self.process_command(&request).await?;
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