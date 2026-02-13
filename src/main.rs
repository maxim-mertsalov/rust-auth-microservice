mod errors;
mod api;
mod db;
mod state;
mod services;
mod repositories;
mod config;
mod models;
mod dto;
mod utils;

use std::sync::Arc;
use actix_web::{web, App, HttpServer};
use actix_web::middleware::Logger;

use log::info;

use crate::api::{auth};
use crate::db::postgres::create_postgres_pool;
use crate::db::redis::create_redis_pool;
use crate::repositories::AppRepositories;
use crate::services::AppServices;
use crate::state::app_state::{AppState};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server...");

    unsafe { std::env::set_var("RUST_LOG", "debug"); }

    let config = config::AppConfig::from_env();
    println!("Loaded env");

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    info!("Logger configured successfully");

    let pg_pool = create_postgres_pool(config.pg_url.as_str(), 5)
        .await
        .expect("Failed to create Postgres pool");
    let shared_pg_pool = Arc::new(pg_pool);

    let redis_pool = create_redis_pool(config.redis_url.as_str(), 5)
        .await
        .expect("Failed to create Redis pool");
    let shared_redis_pool = Arc::new(redis_pool);

    let shared_email_sender = Arc::from(config.email_sender);

    let repositories = AppRepositories::new(shared_pg_pool.clone(), shared_redis_pool.clone());
    let services = AppServices::new(repositories, shared_email_sender);


    let app_state = AppState::builder()
        .with_services(services)
        .with_pg_pool(shared_pg_pool)
        .with_redis_pool(shared_redis_pool)
        .with_secret_key(config.secret_key)
        .with_sudo_secret_key(config.sudo_secret_key)
        .with_cost(8)
        .with_enable_email_reset(config.enable_email_reset)
        .with_require_email_confirmation(config.require_email_confirmation)
        .build().unwrap();

    let shared_app_state = Arc::new(app_state);

    info!("Starting HTTP server on port {}", config.port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Arc::clone(&shared_app_state)))
            .wrap(Logger::new("%a %{User-Agent}i"))
            .service(
                web::scope("/api/v1")
                    .configure(api::routes_config)
            )
    })
        .bind(("0.0.0.0", config.port))?
        .run()
        .await
}