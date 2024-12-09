use crate::prelude::*;
pub mod register;
pub mod login;

pub fn auth_routes_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .service(register::register_user)
            .service(login::login_user)
    );
}