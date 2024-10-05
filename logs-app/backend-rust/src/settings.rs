//! Application settings builder.

use std::net::{IpAddr, Ipv4Addr};

use local_ip_address::local_ip;

#[derive(serde::Deserialize)]
pub struct Settings {
    pub host: String,
    pub port: u16,
    pub local_ip: String,
}

/// Gets the host and port from `settings.yaml`, and the local Ipv4 address from `local_ip()`.
pub fn get_settings() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determinate the current directory");
    let settings_dir = base_path.join("settings.yaml");

    let mut settings = config::Config::builder()
        .add_source(config::File::from(settings_dir))
        .build()?
        .try_deserialize::<Settings>()?;

    settings.local_ip = local_ip().unwrap_or(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 0))
    )
    .to_string();

    Ok(settings)
}