//! Application settings builder.

#[derive(serde::Deserialize)]
pub struct Settings {
    pub host: String,
    pub port: u16,
}

/// Gets the settings from `settings.yaml`.
pub fn get_settings() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determinate the current directory");
    let settings_dir = base_path.join("settings.yaml");

    let settings = config::Config::builder()
        .add_source(config::File::from(settings_dir))
        .build()?;

    settings.try_deserialize::<Settings>()
}