use std::sync::Arc;
use actix_web::{post, web, HttpResponse};
use validator::Validate;
use crate::dto::ApiResponse;
use crate::errors::app_error::AppError;
use crate::state::app_state::AppState;
use crate::dto::auth::signup as signup_dto;
use crate::services::auth::signup::ISignUpService;

#[post("/init")]
async fn signup_init_profile(state: web::Data<Arc<AppState>>, req: web::Json<signup_dto::InitProfileReq>) -> Result<HttpResponse, AppError> {
    // let _ = req.validate()?;

    let sign_up_res = state.get_services().auth_services.signup_service
        .init_session_with_profile(&state, req.0).await?;

    let api_resp: ApiResponse<signup_dto::InitProfileRes> = ApiResponse {
        status: "success".to_string(),
        message: "Sign-up session successfully created!".to_string(),
        data: Some(sign_up_res),
    };

    Ok(HttpResponse::Ok().json(api_resp))
}

#[post("/profile")]
async fn signup_profile(state: web::Data<Arc<AppState>>, req: web::Json<signup_dto::SetProfileReq>) -> Result<HttpResponse, AppError> {
    // let _ = req.validate()?;

    let sign_up_res = state.get_services().auth_services.signup_service
        .set_profile_session(&state, req.0).await?;

    let api_resp: ApiResponse<signup_dto::SetProfileRes> = ApiResponse {
        status: "success".to_string(),
        message: "Profile data set successfully".to_string(),
        data: Some(sign_up_res),
    };

    Ok(HttpResponse::Ok().json(api_resp))
}

#[post("/email")]
async fn signup_email(state: web::Data<Arc<AppState>>, req: web::Json<signup_dto::SetEmailReq>) -> Result<HttpResponse, AppError> {
    req.validate()?;

    let sign_up_res = state.get_services().auth_services.signup_service
        .set_email_session(&state, req.0).await?;

    let api_resp: ApiResponse<signup_dto::SetEmailRes> = ApiResponse {
        status: "success".to_string(),
        message: "Email set successfully".to_string(),
        data: Some(sign_up_res),
    };

    Ok(HttpResponse::Ok().json(api_resp))
}

#[post("/email/resend")]
async fn signup_resend_code(state: web::Data<Arc<AppState>>, req: web::Json<signup_dto::ResendEmailCodeReq>) -> Result<HttpResponse, AppError> {
    // let _ = req.validate()?;

    let _sign_up_res = state.get_services().auth_services.signup_service
        .resend_email_verification(&state, req.0).await?;

    let api_resp: ApiResponse<signup_dto::ResendEmailCodeRes> = ApiResponse {
        status: "success".to_string(),
        message: "Email code resent successfully".to_string(),
        data: None,
    };

    Ok(HttpResponse::Ok().json(api_resp))
}

#[post("/email/verify")]
async fn signup_email_verify(state: web::Data<Arc<AppState>>, req: web::Json<signup_dto::VerifyEmailReq>) -> Result<HttpResponse, AppError> {
    // let _ = req.validate()?;

    let sign_up_res = state.get_services().auth_services.signup_service
        .verify_email_session(&state, req.0).await?;

    let api_resp: ApiResponse<signup_dto::VerifyEmailRes> = ApiResponse {
        status: "success".to_string(),
        message: "Email verified successfully".to_string(),
        data: Some(sign_up_res),
    };

    Ok(HttpResponse::Ok().json(api_resp))
}

#[post("/password")]
async fn signup_password(state: web::Data<Arc<AppState>>, req: web::Json<signup_dto::SetPasswordReq>) -> Result<HttpResponse, AppError> {
    req.validate()?;

    let sign_up_res = state.get_services().auth_services.signup_service
        .set_password_session(&state, req.0).await?;

    let api_resp: ApiResponse<signup_dto::SetPasswordRes> = ApiResponse {
        status: "success".to_string(),
        message: "Password set successfully".to_string(),
        data: Some(sign_up_res),
    };

    Ok(HttpResponse::Ok().json(api_resp))
}

#[post("/final")]
async fn signup_finalise(state: web::Data<Arc<AppState>>, req: web::Json<signup_dto::FinalizeSignUpReq>) -> Result<HttpResponse, AppError> {
    // let _ = req.validate()?;

    let sign_up_res = state.get_services().auth_services.signup_service
        .finalise_session(&state, req.0).await?;

    let api_resp: ApiResponse<signup_dto::FinalizeSignUpRes> = ApiResponse {
        status: "success".to_string(),
        message: "User created successfully".to_string(),
        data: Some(sign_up_res),
    };

    Ok(HttpResponse::Ok().json(api_resp))
}

#[post("/back")]
async fn signup_back(state: web::Data<Arc<AppState>>, req: web::Json<signup_dto::ReturnBackSessionReq>) -> Result<HttpResponse, AppError> {
    // let _ = req.validate()?;

    let sign_up_res = state.get_services().auth_services.signup_service
        .return_back(&state, req.0).await?;

    let api_resp: ApiResponse<signup_dto::ReturnBackSessionRes> = ApiResponse {
        status: "success".to_string(),
        message: "You have returned successfully".to_string(),
        data: Some(sign_up_res),
    };

    Ok(HttpResponse::Ok().json(api_resp))
}

pub fn signup_service(cfg: &mut web::ServiceConfig) {
    cfg
        .service(
            web::scope("/signup")
                .service(signup_init_profile)
                .service(signup_profile)
                .service(signup_email)
                .service(signup_resend_code)
                .service(signup_email_verify)
                .service(signup_password)
                .service(signup_finalise)
                .service(signup_back)
        );
}