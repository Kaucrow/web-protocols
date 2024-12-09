use crate::prelude::*;
use crate::{ Settings, get_postgres_conn_str };
use crate::app::routes;
use anyhow::Result;

pub struct AppServer {
    host: String,
    port: u16,
    server: actix_web::dev::Server,
}

impl AppServer {
    pub async fn build(
        settings: Settings,
    ) -> Result<Self> {
        let db_conn_str = get_postgres_conn_str()?;

        let db = PgPoolOptions::new()
            .max_connections(5)
            .connect(&db_conn_str)
            .await?;

        let host = settings.app.host.clone();
        let port = settings.app.port;

        let server = run(db, settings).await?;

        Ok(Self { host, port, server })
    }

    pub fn host(&self) -> &String {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        tracing::info!(target: "app", "App server listening on {}:{}", self.host(), self.port());
        self.server.await
    }
}

async fn run(
    db: PgPool,
    settings: Settings,
) -> Result<actix_web::dev::Server> {
    let db = actix_web::web::Data::new(db);

    let server = actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .wrap(
                actix_cors::Cors::default()
                .allowed_origin("127.0.0.1")
                .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE", "OPTIONS"])
                .allowed_headers(vec![
                    actix_web::http::header::AUTHORIZATION,
                    actix_web::http::header::ACCEPT,
                    actix_web::http::header::CONTENT_TYPE,
                ])
                .expose_headers(&[actix_web::http::header::CONTENT_TYPE])
                .supports_credentials()
                .max_age(3600)
            )
            .app_data(db.clone())
            .configure(routes::auth::auth_routes_config)
            .wrap(actix_web::middleware::NormalizePath::trim())
    });

    let server = {
        let address = format!(
            "{}:{}",
            settings.app.host, settings.app.port
        );
        let listener = std::net::TcpListener::bind(&address)?;
        server.listen(listener)?
        .run()
    };

    Ok(server)
}