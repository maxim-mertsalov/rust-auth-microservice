use std::sync::Arc;
use actix_web::{delete, post, web, HttpResponse};
use actix_web::http::StatusCode;
use crate::dto::{ApiErrorResponse, ApiResponse};
use crate::dto::auth::sessions::{CheckTokenReq, GetAllSessionsReq, LogoutReq, RefreshTokenReq, RefreshTokenRes, TerminateAllOtherSessionsReq, TerminateSessionReq};
use crate::errors::app_error::AppError;
use crate::models::auth::sessions::{MinFieldsSession, Session};
use crate::services::auth::session::ISessionService;
use crate::state::app_state::AppState;

#[post("/refresh")]
async fn refresh_token(state: web::Data<Arc<AppState>>, req: web::Json<RefreshTokenReq>) -> Result<HttpResponse, AppError> {
    // let device_info = Converter::new_device_info(http_req);

    let res = state.get_services().auth_services.session_service.refresh_tokens(&state, req.0).await?;

    let api_resp: ApiResponse<RefreshTokenRes> = ApiResponse {
        status: "success".to_string(),
        message: "Token refreshed successfully!".to_string(),
        data: Some(res),
    };

    Ok(HttpResponse::Ok().json(api_resp))
}

#[post("/check")]
async fn check_token(state: web::Data<Arc<AppState>>, req: web::Json<CheckTokenReq>) -> Result<HttpResponse, AppError> {
    let res = state.get_services().auth_services.session_service.check_user(&state, req.access_token.clone()).await?;

    let api_resp: ApiResponse<RefreshTokenRes> = ApiResponse {
        status: "success".to_string(),
        message: "Access token is not expired".to_string(),
        data: None,
    };

    let err_resp = ApiErrorResponse {
        status: "error".to_string(),
        message: "Access token is expired!".to_string(),
        errors: None,
    };

    if res {
        return Err(AppError::Custom((err_resp, StatusCode::UNAUTHORIZED)));
    }

    Ok(HttpResponse::Ok().json(api_resp))
}

#[delete("/logout")]
pub async fn logout_user(state: web::Data<Arc<AppState>>, req: web::Json<LogoutReq>) -> Result<HttpResponse, AppError> {
    state.get_services().auth_services.session_service.logout_user(&state, req.0).await?;

    let api_resp: ApiResponse<String> = ApiResponse {
        status: "success".to_string(),
        message: "User logged out successfully!".to_string(),
        data: None,
    };

    Ok(HttpResponse::Ok().json(api_resp))
}

#[post("/sessions")]
pub async fn get_sessions(state: web::Data<Arc<AppState>>, req: web::Json<GetAllSessionsReq>) -> Result<HttpResponse, AppError> {
    let res = state.get_services().auth_services.session_service.get_all_sessions(&state, req.0).await?;

    let api_resp: ApiResponse<Vec<MinFieldsSession>> = ApiResponse {
        status: "success".to_string(),
        message: "All sessions got successfully".to_string(),
        data: Some(res),
    };

    Ok(HttpResponse::Ok().json(api_resp))
}

#[delete("/sessions")]
pub async fn terminate_session(state: web::Data<Arc<AppState>>, req: web::Json<TerminateSessionReq>) -> Result<HttpResponse, AppError> {
    state.get_services().auth_services.session_service.terminate_session(&state, req.0).await?;

    let api_resp: ApiResponse<String> = ApiResponse {
        status: "success".to_string(),
        message: "Session terminated successfully!".to_string(),
        data: None,
    };

    Ok(HttpResponse::Ok().json(api_resp))
}

#[delete("/sessions/all")]
pub async fn terminate_all_other_sessions(state: web::Data<Arc<AppState>>, req: web::Json<TerminateAllOtherSessionsReq>) -> Result<HttpResponse, AppError> {
    state.get_services().auth_services.session_service.terminate_all_other_sessions(&state, req.0).await?;

    let api_resp: ApiResponse<String> = ApiResponse {
        status: "success".to_string(),
        message: "All other sessions terminated successfully!".to_string(),
        data: None,
    };

    Ok(HttpResponse::Ok().json(api_resp))
}

pub fn sessions_service(cfg: &mut web::ServiceConfig) {
    cfg
        .service(refresh_token)
        .service(check_token)
        .service(logout_user)
        .service(get_sessions)
        .service(terminate_session)
        .service(terminate_all_other_sessions)
    ;
}