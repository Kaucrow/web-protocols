use crate::prelude::*;

#[derive(Deserialize)]
pub struct SmtpSettings {
    pub tls: bool,
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize)]
pub struct PostfixSettings {
    pub host: String,
}

#[derive(Deserialize)]
pub struct PostgresSettings {
    pub user: String,
    pub password: String,
    pub host: String,
    pub db_name: String,
    pub require_ssl: bool,
}

#[derive(Deserialize)]
pub struct AppSettings {
    pub protocol: String,
    pub host: String,
    pub port: u16
}

#[derive(Deserialize)]
pub struct Settings {
    pub domain: String,
    pub smtp: SmtpSettings,
    pub app: AppSettings,
    pub postfix: PostfixSettings,
    pub postgres: PostgresSettings,
    pub debug: bool,
}

pub enum Environment {
    Development,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Development => "development",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "development" => Ok(Self::Development),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `development` or `production`.",
                other
            )),
        }
    }
}

pub fn get_settings() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let settings_directory = base_path.join("settings");

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or("development".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");

    let environment_filename: String = format!("{}.yaml", environment.as_str());
    let settings = config::Config::builder()
        .add_source(config::File::from(settings_directory.join("base.yaml")))
        .add_source(config::File::from(
            settings_directory.join(settings_directory.join(environment_filename))
        ))
        .build()?;

    settings.try_deserialize::<Settings>()
}