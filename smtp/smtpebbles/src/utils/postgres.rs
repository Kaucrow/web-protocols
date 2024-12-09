use crate::get_settings;
use anyhow::Result;

pub fn get_postgres_conn_str() -> Result<String> {
    let settings = get_settings()?;

    let postgres_conn_str = {
        let settings = &settings.postgres;

        let mut conn_str = format!(
            "postgresql://{}:{}@{}/{}",
            settings.user, settings.password, settings.host, settings.db_name
        );

        if settings.require_ssl {
            conn_str += "?sslmode=require";
        }

        conn_str
    };

    Ok(postgres_conn_str)
}