use tokio::{
    net::{ TcpListener, TcpStream },
    fs::File,
    io::{ AsyncReadExt, AsyncWriteExt },
};
use std::{
    error::Error,
    net::SocketAddr,
    path::Path,
};
use anyhow::{ anyhow, Result, bail };

async fn handle_client(mut cmd_socket: TcpStream) -> Result<()> {
    let welcome_msg = "220 Welcome to Rust FTP Server\r\n";
    cmd_socket.write_all(welcome_msg.as_bytes()).await?;

    let mut dat_socket: Option<TcpListener> = None;

    let mut buf = vec![0; 1024];

    loop {
        let n = cmd_socket.read(&mut buf).await?;

        if n == 0 {
            break;
        }

        let command = String::from_utf8_lossy(&buf[..n]);
        println!("Received command: {}", command);

        if command.starts_with("USER") {
            cmd_socket.write_all(b"331 Username okay, need password.\r\n").await?;
        } else if command.starts_with("PASS") {
            cmd_socket.write_all(b"230 User logged in, proceed.\r\n").await?;
        } else if command.starts_with("SYST") {
            #[cfg(target_os = "windows")]
            cmd_socket.write_all(b"215 Windows_NT\r\n").await?;

            #[cfg(not(target_os = "windows"))]
            cmd_socket.write_all(b"215 UNIX Type: L8\r\n").await?;
        } else if command.starts_with("FEAT") {
            cmd_socket.write_all(b"211 Features:\r\n").await?;
            cmd_socket.write_all(b"UTF8\r\n").await?;
            cmd_socket.write_all(b"211 End\r\n").await?;
        } else if command.starts_with("PWD") {
            cmd_socket.write_all(b"257 \"/\" is the current directory\r\n").await?;
        } else if command.starts_with("PASV") {
            let listener = handle_pasv_command(&mut cmd_socket).await?;
            dat_socket = Some(listener);
        } else if command.starts_with("TYPE") {
            let mode = command.split_whitespace().nth(1).unwrap_or("");
            match mode {
                "A" => {
                    cmd_socket.write(b"200 Type set to A\r\n").await?;
                }
                "I" => {
                    cmd_socket.write(b"200 Type set to I\r\n").await?;
                }
                _ => {
                    cmd_socket.write(b"504 Command not implemented for that parameter\r\n").await?;
                }
            }
        } else if command.starts_with("LIST") {
            let listing = list_files(".")?;
            //let listing = String::from("drwxr-xr-x            1            0            0              0 Sep 12 19:48 WinSAT");
            println!("LISTING: {}", listing);
            if let Some(socket) = &mut dat_socket {
                println!("ATTEMPTING WRITE");
                cmd_socket.write_all(b"150 Sending directory list\r\n").await?;
                let (mut data_stream, _) = socket.accept().await?;
                data_stream.write_all(listing.as_bytes()).await?;
                println!("WROTE");
            } else {
                cmd_socket.write_all(b"550 Pasv mode not set.\r\n").await?;
            }
        } else if command.starts_with("RETR") {
            let filename = command.split_whitespace().nth(1).unwrap_or("");
            let file_path = Path::new(filename);

            if file_path.exists() && file_path.is_file() {
                cmd_socket.write_all(b"150 Opening data connection for file transfer.\r\n").await?;

                let mut file = File::open(file_path).await?;
                let mut file_buf = Vec::new();

                file.read_to_end(&mut file_buf).await?;

                cmd_socket.write_all(&file_buf).await?;

                cmd_socket.write_all(b"226 Transfer complete.\r\n").await?;
            } else {
                cmd_socket.write_all(b"550 File not found.\r\n").await?;
            }
        } else if command.starts_with("QUIT") {
            cmd_socket.write_all(b"221 Goodbye.\r\n").await?;
            break;
        } else {
            cmd_socket.write_all(b"502 Command not implemented.\r\n").await?;
        }
    }

    Ok(())
}

async fn handle_pasv_command(cmd_socket: &mut TcpStream) -> Result<TcpListener> {
    use rand;
    let passive_port = 50000 + rand::random::<u8>() as u16 % 10;  // Random port between 50000 and 50010
    
    // Open a TCP listener on an ephemeral port for passive mode
    let listener = TcpListener::bind(format!("127.0.0.1:{}", passive_port)).await?;
    let local_addr = listener.local_addr()?;

    println!("LOCAL ADDR: {}", local_addr);

    // Extract IP and port components for PASV response
    let ip = local_addr.ip();
    let port = local_addr.port();

    // Send PASV response with IP and port information
    let response = format_pasv_response(ip, port);

    println!("PASV RESPONSE: {}", &response);

    cmd_socket.write_all(response.as_bytes()).await?;

    // Wait for the client to connect to the passive port
    if let Ok((mut data_stream, _)) = listener.accept().await {
        // At this point, `data_stream` is ready for file transfers
        println!("Client connected for passive mode transfer");
        let listing = list_files(".")?;
        cmd_socket.write_all(b"150 Sending directory list\r\n").await?;
        data_stream.write_all(listing.as_bytes()).await?;
        println!("WROTE");
        cmd_socket.write_all(b"226 Directory send ok\r\n").await?;
        // You would use `data_stream` to send/receive files
    } else {
        bail!("Failed to establish passive mode connection.");
    }

    Ok(listener)
    /*// Wait for the client to connect to the passive port
    if let Ok((data_stream, _)) = listener.accept().await {
        // At this point, `data_stream` is ready for file transfers
        println!("Client connected for passive mode transfer");

        Ok(data_stream)
        // You would use `data_stream` to send/receive files
    } else {
        bail!("Failed to establish passive mode connection.");
    }*/
}

fn format_pasv_response(ip: std::net::IpAddr, port: u16) -> String {
    // Separate the port into two 8-bit values
    let p1 = (port / 256) as u8;
    let p2 = (port % 256) as u8;

    match ip {
        std::net::IpAddr::V4(ipv4) => {
            let octets = ipv4.octets();
            format!("227 Entering Passive Mode ({},{},{},{},{},{})\r\n",
                    octets[0], octets[1], octets[2], octets[3], p1, p2)
        },
        std::net::IpAddr::V6(_) => {
            // FTP does not directly support IPv6 in PASV mode, but this is how you might handle it
            // if you need to handle an IPv6 address specifically for your server.
            // Returning an error or handling it accordingly might be better.
            "522 Network protocol not supported (IPv6 is not supported in PASV mode)\r\n".to_string()
        }
    }
}

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Local};

/// List files in a directory and format for FTP-style directory listing (compatible with WinSCP)
fn list_files(dir: &str) -> Result<String> {
    let mut listing = String::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let metadata = entry.metadata()?;

        // Get file type (directory or file)
        let is_dir = metadata.is_dir();
        let file_type = if is_dir { "drwxr-xr-x" } else { "-rw-r--r--" }; // Simplified permissions

        let hard_links = 0;

        let user_uid = 0;

        let group_gid = 0;

        // Get owner and group (using placeholders, as these are also platform-specific)
        let owner = "user"; // Placeholder for owner name
        let group = "group"; // Placeholder for group name

        // Get file size
        let file_size = metadata.len();

        // Get last modified time (formatted as `Month Day Hour:Minute:Second`)
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
            file_type, hard_links, user_uid, group_gid, file_size, timestamp, file_name
        ));
    }

    listing.push_str("226 Directory send OK.\r\n");
    Ok(listing)
}


async fn run_server(addr: SocketAddr) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("Server listening on {}", addr);

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket).await {
                eprintln!("Error handling client: {}", e);
            }
        });
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr: SocketAddr = "127.0.0.1:21".parse()?;
    run_server(addr).await?;
    Ok(())
}