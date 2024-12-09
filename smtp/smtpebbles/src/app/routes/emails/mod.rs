pub mod send;

use crate::prelude::*;

pub fn emails_routes_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/emails")
            .service(send::send_email)
    );
}