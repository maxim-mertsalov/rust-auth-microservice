use actix_web::web;
use crate::api::auth::sessions::sessions_service;
use crate::api::auth::signin::signin_service;
use crate::api::auth::signup::signup_service;
use crate::api::auth::user_management::user_management_service;

pub mod signup;
pub mod signin;
mod sessions;
mod user_management;

pub fn auth_services(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .configure(signup_service)
            .configure(signin_service)
            .configure(sessions_service)
            .configure(user_management_service)
    );
}