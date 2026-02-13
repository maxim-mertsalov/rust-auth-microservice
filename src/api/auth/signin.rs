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

// Select sign-in method
#[post("/method")]
async fn signin_get_auth_methods(state: web::Data<Arc<AppState>>, req: web::Json<signin_dto::GetAuthMethodsReq>) -> Result<HttpResponse, AppError> {
    // let _ = req.validate()?;

    let sign_up_res = state.get_services().auth_services.signin_service
        .get_auth_methods(&state, req.0).await?;

    let api_resp: ApiResponse<signin_dto::GetAuthMethodsRes> = ApiResponse {
        status: "success".to_string(),
        message: "Methods got successfully".to_string(),
        data: Some(sign_up_res),
    };

    Ok(HttpResponse::Ok().json(api_resp))
}

#[post("/method/select")]
async fn signin_select_auth_methods(state: web::Data<Arc<AppState>>, req: web::Json<signin_dto::SelectAuthMethodReq>) -> Result<HttpResponse, AppError> {
    // let _ = req.validate()?;

    let sign_up_res = state.get_services().auth_services.signin_service
        .select_auth_method(&state, req.0).await?;

    let api_resp: ApiResponse<signin_dto::SelectAuthMethodRes> = ApiResponse {
        status: "success".to_string(),
        message: "Method set successfully".to_string(),
        data: Some(sign_up_res),
    };

    Ok(HttpResponse::Ok().json(api_resp))
}

// 1. Password
#[post("/password")]
async fn signin_password(state: web::Data<Arc<AppState>>, req: web::Json<signin_dto::SetPasswordReq>) -> Result<HttpResponse, AppError> {
    // let _ = req.validate()?;

    let sign_up_res = state.get_services().auth_services.signin_service
        .set_password(&state, req.0).await?;

    let api_resp: ApiResponse<signin_dto::SetPasswordRes> = ApiResponse {
        status: "success".to_string(),
        message: "Password is verified successfully".to_string(),
        data: Some(sign_up_res),
    };

    Ok(HttpResponse::Ok().json(api_resp))
}

// 2. Recovery email
#[post("/recovery-email")]
async fn signin_get_recovery_emails(state: web::Data<Arc<AppState>>, req: web::Json<signin_dto::GetRecoveryEmailsReq>) -> Result<HttpResponse, AppError> {
    let sign_up_res = state.get_services().auth_services.signin_service
        .get_recovery_emails(&state, req.0).await?;

    let api_resp: ApiResponse<signin_dto::GetRecoveryEmailsRes> = ApiResponse {
        status: "success".to_string(),
        message: "Recovery emails got successfully".to_string(),
        data: Some(sign_up_res),
    };

    Ok(HttpResponse::Ok().json(api_resp))
}

#[post("/recovery-email/select")]
async fn signin_select_recovery_email(state: web::Data<Arc<AppState>>, req: web::Json<signin_dto::SelectRecoveryEmailReq>) -> Result<HttpResponse, AppError> {
    let sign_up_res = state.get_services().auth_services.signin_service
        .select_recovery_email(&state, req.0).await?;

    let api_resp: ApiResponse<signin_dto::SelectRecoveryEmailRes> = ApiResponse {
        status: "success".to_string(),
        message: "Code was send to recovery email".to_string(),
        data: Some(sign_up_res),
    };

    Ok(HttpResponse::Ok().json(api_resp))
}

#[post("/recovery-email/verify")]
async fn signin_verify_recovery_email(state: web::Data<Arc<AppState>>, req: web::Json<signin_dto::VerifyRecoveryEmailCodeReq>) -> Result<HttpResponse, AppError> {
    let sign_up_res = state.get_services().auth_services.signin_service
        .verify_recovery_email_code(&state, req.0).await?;

    let api_resp: ApiResponse<signin_dto::VerifyRecoveryEmailCodeRes> = ApiResponse {
        status: "success".to_string(),
        message: "Recovery email is verified successfully".to_string(),
        data: Some(sign_up_res),
    };

    Ok(HttpResponse::Ok().json(api_resp))
}

// 3. Recovery code
#[post("/recovery-code")]
async fn signin_set_recovery_code(state: web::Data<Arc<AppState>>, req: web::Json<signin_dto::SetRecoveryCodeReq>) -> Result<HttpResponse, AppError> {
    let sign_up_res = state.get_services().auth_services.signin_service
        .set_recovery_code(&state, req.0).await?;

    let api_resp: ApiResponse<signin_dto::SetRecoveryCodeRes> = ApiResponse {
        status: "success".to_string(),
        message: "Recovery code is verified successfully".to_string(),
        data: Some(sign_up_res),
    };

    Ok(HttpResponse::Ok().json(api_resp))
}


// 4. Email verification
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

// email code + recovery code
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

#[post("/final")]
async fn signin_finalise(state: web::Data<Arc<AppState>>, req: web::Json<signin_dto::FinalizeSignInReq>) -> Result<HttpResponse, AppError> {
    // let _ = req.validate()?;

    let sign_up_res = state.get_services().auth_services.signin_service
        .final_session(&state, req.0).await?;

    let api_resp: ApiResponse<signin_dto::FinalizeSignInRes> = ApiResponse {
        status: "success".to_string(),
        message: "Sign-In finished successfully".to_string(),
        data: Some(sign_up_res),
    };

    Ok(HttpResponse::Ok().json(api_resp))
}

pub fn signin_service(cfg: &mut web::ServiceConfig) {
    cfg
        .service(
            web::scope("/signin")
                .service(signin_init_email)

                .service(signin_get_auth_methods)
                .service(signin_select_auth_methods)
                
                // password
                .service(signin_password)

                // recovery email
                .service(signin_get_recovery_emails)
                .service(signin_select_recovery_email)
                .service(signin_verify_recovery_email)

                // recovery code
                .service(signin_set_recovery_code)

                // email verification
                .service(signin_verify_email)

                // resend email code
                .service(signin_resend_email_code)

                .service(signin_finalise)
        );
}