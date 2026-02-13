use std::sync::Arc;
use actix_web::{get, post, web, HttpResponse};
use crate::errors::app_error::AppError;
use crate::state::app_state::AppState;
use crate::utils::converter;
use crate::utils::hash::Hasher;

#[get("/")]
async fn hello() -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().body("Server works fine!"))
}

#[post("/echo")]
async fn echo(req_body: String) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().body(req_body))
}

#[post("/test/bcrypt")]
async fn test_bcrypt(req_body: String, state: web::Data<Arc<AppState>>) -> Result<HttpResponse, AppError> {
    let res_with_salt = converter::Converter::hash_string(&req_body);
    let res_without_salt = bcrypt::hash(&req_body, state.get_cost())?;

    let res = format!("With salt: {}\nWithout salt: {}", res_with_salt, res_without_salt);

    Ok(HttpResponse::Ok().body(res))
}

#[post("/test/argon2")]
async fn test_argon2(req_body: String, state: web::Data<Arc<AppState>>) -> Result<HttpResponse, AppError> {
    let res_without_salt = Hasher::hash_password(req_body).await?;

    let res = format!("Argon2: {}", res_without_salt);

    Ok(HttpResponse::Ok().body(res))
}

#[get("/test/refresh")]
async fn refresh_test(_: web::Data<Arc<AppState>>) -> Result<HttpResponse, AppError> {
    let refresh_test = crate::utils::access_tokens::TokenBuilder::generate_refresh_token();

    let msg = format!("Refresh token: {}", refresh_test);

    Ok(HttpResponse::Ok().body(msg))
}

#[get("/test/postgres")]
async fn postgres_echo(state: web::Data<Arc<AppState>>) -> Result<HttpResponse, AppError> {
    match state.get_services().test_service.test_postgres().await {
        Ok(val) => Ok(HttpResponse::Ok().body(format!("All is ok: {}", val))),
        Err(e) => Err(e.into()),

    }
}

#[get("/test/redis")]
async fn redis_echo(state: web::Data<Arc<AppState>>) -> Result<HttpResponse, AppError> {
    match state.get_services().test_service.test_redis().await {
        Ok(val) => Ok(HttpResponse::Ok().body(format!("All is ok: {}", val))),
        Err(e) => Err(e.into()),
    }
}

#[get("/test/email")]
async fn email_test(state: web::Data<Arc<AppState>>) -> Result<HttpResponse, AppError> {
    match state.get_services().test_service.test_email_sender().await {
        Ok(val) => Ok(HttpResponse::Ok().body(format!("All is ok: {}", val))),
        Err(e) => Err(e.into()),
    }
}

pub fn test_services(cfg: &mut web::ServiceConfig) {
    cfg
        .service(hello)
        .service(echo)
        .service(test_bcrypt)
        .service(test_argon2)
        .service(refresh_test)
        .service(email_test)
        .service(redis_echo)
        .service(postgres_echo);
}