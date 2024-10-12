//! A UDP server.

use crate::{
    ClientInfo,
    ServerOrigin,
    has_init,
    common::handle_frame,
    settings::Settings,
};

/// Starts the server.
pub async fn run(settings: Settings) -> std::io::Result<()> {
    let sock = actix_web::rt::net::UdpSocket::bind(format!("{}:{}", settings.host, settings.udp_port)).await?;
    let mut buf = [0; 1024];

    loop {
        match sock.recv_from(&mut buf).await {
            Ok((len, addr)) => {
                tracing::debug!("{:?} bytes received from {:?}", len, addr);

                let msg = { 
                    let msg= String::from_utf8_lossy(&buf[..len]).to_string();
                    msg.trim().to_string()
                };

                if has_init(msg.as_str()) {
                    let client = ClientInfo::from(addr);
                    handle_frame(client, msg, ServerOrigin::Udp);
                }
            }
            Err(e) => {
                tracing::error!(target: "backend", "Error receiving UDP message: {}", e);
            }
        }
    }
}