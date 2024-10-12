use std::net::{ IpAddr, SocketAddr };
use actix_web::HttpRequest;
use dyn_fmt::AsStrFormatExt;

/// Holds the ip and port for an incoming connection
#[derive(Debug, Clone)]
pub struct ClientInfo {
    ip: IpAddr,
    port: u16,
}

impl From<&HttpRequest> for ClientInfo {
    fn from(req: &HttpRequest) -> Self {
        let addr = req.peer_addr().unwrap_or_else(|| SocketAddr::from(([0, 0, 0, 0], 0)));
        ClientInfo {
            ip: addr.ip(),
            port: addr.port(),
        }
    }
}

impl From<SocketAddr> for ClientInfo {
    fn from(addr: SocketAddr) -> Self {
        ClientInfo {
            ip: addr.ip(),
            port: addr.port(),
        } 
    }
}

impl ClientInfo {
    pub fn ip(&self) -> IpAddr {
        self.ip
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

/// Log frame.
struct Frame {
    cmd: String,
    data: String,
}

impl TryFrom<String> for Frame {
    type Error = String;

    /// `str` should be formatted as `init^cmd^data^endData^close`.
    fn try_from(str: String) -> Result<Self, Self::Error> {
        // Checks if the frame field matches the expected field
        fn check_field(field: &str, expected: &str, err: &str) -> Result<(), String> {
            if field != expected {
                Err(err.format(&[field, expected]))
            } else {
                Ok(())
            }
        }

        let err = "Malformed frame str: found `{}` instead of `{}`";

        let fields: Vec<&str> = str.split('^').collect();

        let [init, cmd, data, end_data, close] = fields[..] else {
            return Err("Malformed frame str: unexpected number of fields".to_string());
        };

        check_field(init, "init", err)?;
        check_field(end_data, "endData", err)?;
        check_field(close, "close", err)?;

        Ok(Frame { cmd: cmd.to_string(), data: data.to_string() })
    }
}

pub fn has_init(msg: &str) -> bool {
    if let Some(idx) = msg.find('^') {
        if &msg[0..idx] == "init" {
            return true;
        }
    }
    false
}

#[tracing::instrument(name="Handling frame")]
pub fn handle_frame(client: ClientInfo, frame: String) {
    tracing::debug!(target: "backend", "Client {}:{} sent frame: {frame}", client.ip(), client.port());
    match Frame::try_from(frame) {
        Ok(frame) => {
            const TGT: &'static str = "backend-file";

            let message =
                format!(
                    "Received frame from {}:{} [ cmd: {}, data: {} ]",
                    client.ip(), client.port(), frame.cmd, frame.data
                );

            match frame.cmd.to_uppercase().as_str() {
                "DEBUG" => tracing::debug!(target: TGT, message),
                "INFO" => tracing::info!(target: TGT, message),
                "WARN" => tracing::warn!(target: TGT, message),
                "ERROR" => tracing::error!(target: TGT, message),
                _ => tracing::trace!(target: TGT, message),
            }
        }
        Err(e) =>
            tracing::error!(target: "backend", e)
    }
}