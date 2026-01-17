use std::sync::Arc;
use actix_web::{post, web, HttpResponse};
use validator::Validate;
use crate::dto::ApiResponse;
use crate::dto::auth::signin as signin_dto;
use crate::errors::app_error::AppError;
use crate::services::auth::signin::ISignInService;
use crate::state::app_state::AppState;

#[post("/init")]
async fn signin_init_email(state: web::Data<Arc<AppState>>, req: web::Json<signin_dto::InitEmailReq>) -> Result<HttpResponse, AppError> {
    req.validate()?;

    let sign_up_res = state.get_services().auth_services.signin_service
        .init_with_email(&state, req.0).await?;

    let api_resp: ApiResponse<signin_dto::InitEmailRes> = ApiResponse {
        status: "success".to_string(),
        message: "Sign-in session successfully created!".to_string(),
        data: Some(sign_up_res),
    };

    Ok(HttpResponse::Ok().json(api_resp))
}

#[post("/password")]
async fn signin_password(state: web::Data<Arc<AppState>>, req: web::Json<signin_dto::SetPasswordReq>) -> Result<HttpResponse, AppError> {
    // let _ = req.validate()?;

    let sign_up_res = state.get_services().auth_services.signin_service
        .set_password(&state, req.0).await?;

    let api_resp: ApiResponse<signin_dto::SetPasswordRes> = ApiResponse {
        status: "success".to_string(),
        message: "Password verified successfully".to_string(),
        data: Some(sign_up_res),
    };

    Ok(HttpResponse::Ok().json(api_resp))
}

#[post("/email/resend")]
async fn signin_resend_email_code(state: web::Data<Arc<AppState>>, req: web::Json<signin_dto::ResendCodeReq>) -> Result<HttpResponse, AppError> {
    // let _ = req.validate()?;

    let _sign_up_res = state.get_services().auth_services.signin_service
        .resend_code(&state, req.0).await?;

    let api_resp: ApiResponse<signin_dto::ResendCodeRes> = ApiResponse {
        status: "success".to_string(),
        message: "Email code resent successfully".to_string(),
        data: None,
    };

    Ok(HttpResponse::Ok().json(api_resp))
}

#[post("/email/verify")]
async fn signin_verify_email(state: web::Data<Arc<AppState>>, req: web::Json<signin_dto::VerifyEmailReq>) -> Result<HttpResponse, AppError> {
    // let _ = req.validate()?;

    let sign_up_res = state.get_services().auth_services.signin_service
        .verify_email(&state, req.0).await?;

    let api_resp: ApiResponse<signin_dto::VerifyEmailRes> = ApiResponse {
        status: "success".to_string(),
        message: "Email is verified successfully".to_string(),
        data: Some(sign_up_res),
    };

    Ok(HttpResponse::Ok().json(api_resp))
}

#[post("/final")]
async fn signin_finalise(state: web::Data<Arc<AppState>>, req: web::Json<signin_dto::FinalizeSignInReq>) -> Result<HttpResponse, AppError> {
    // let _ = req.validate()?;

    let sign_up_res = state.get_services().auth_services.signin_service
        .finalise_session(&state, req.0).await?;

    let api_resp: ApiResponse<signin_dto::FinalizeSignInRes> = ApiResponse {
        status: "success".to_string(),
        message: "Email is verified successfully".to_string(),
        data: Some(sign_up_res),
    };

    Ok(HttpResponse::Ok().json(api_resp))
}

pub fn signin_service(cfg: &mut web::ServiceConfig) {
    cfg
        .service(
            web::scope("/signin")
                .service(signin_init_email)
                .service(signin_password)
                .service(signin_resend_email_code)
                .service(signin_verify_email)
                .service(signin_finalise)
        );
}