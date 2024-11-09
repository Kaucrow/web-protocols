use crate::prelude::*;
use anyhow::Result;

#[derive(Clone)]
pub struct FtpServer {
    pub host: String,
    pub port: u16,
    pub base_dir: PathBuf,
}

#[derive(PartialEq)]
pub enum TransferType {
    Ascii,
    Binary,
}

pub struct FtpSession {
    pub server: FtpServer,

    pub real_dir: PathBuf,
    pub virtual_dir: String,

    pub ctrl: TcpStream,
    pub data: Option<TcpListener>,

    pub transfer_type: TransferType,
}

impl FtpServer {
    pub fn new(host: &'static str, port: u16, base_dir: &'static str) -> Result<Self> {
        Ok(Self {
            host: host.to_string(),
            port,
            base_dir: Path::new(base_dir).to_path_buf(),
        })
    }

    fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub async fn run(&self) -> Result<()> {
        let addr = self.addr();
        let listener = TcpListener::bind(&addr).await?;
        tracing::info!("FTP server listening on {}", &addr);

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
            real_dir: base_dir,
            virtual_dir: String::from("/"),
            ctrl: stream,
            data: None,
            transfer_type: TransferType::Ascii,
        }
    }

    // Run function to handle the session lifecycle
    async fn run(mut self) -> Result<()> {
        // Send a welcome message
        self.send_response("220 Welcome to the Rust FTP Server\r\n").await?;

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