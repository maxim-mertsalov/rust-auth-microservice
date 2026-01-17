use std::error::Error;
use actix_web::{web, HttpResponse, HttpResponseBuilder, Responder, ResponseError};
use crate::api::auth::auth_services;
use crate::api::test::test_services;
use crate::dto::ApiErrorResponse;

pub mod auth;
pub mod test;

async fn default_route() -> impl Responder {
    let res = ApiErrorResponse {
        status: "error".to_string(),
        message: "Not Found".to_string(),
        errors: None,
    };

    HttpResponse::NotFound().json(res)
}

// pub fn configure_json_errors(cfg: &mut web::ServiceConfig) {
//     cfg.app_data(web::JsonConfig::default().error_handler(|err, _req| {
//         actix_web::error::InternalError::from_response(
//             err.cause(),
//             HttpResponseBuilder::new(err.status_code()).json(ApiErrorResponse {
//                 status: "error".to_string(),
//                 message: err.description().to_string(),
//                 errors: None,
//             }),
//         )
//             .into()
//     }));
// }

pub fn routes_config(cfg: &mut web::ServiceConfig) {
    cfg
        .configure(auth_services)
        .configure(test_services)
        // .configure(configure_json_errors)
        .default_service(web::to(default_route));
    ;
}


