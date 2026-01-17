use std::sync::Arc;
use actix_web::{delete, get, put, web, HttpResponse};
use crate::dto::ApiResponse;
use crate::dto::auth::user_management::{DeleteUserReq, UpdateUserReq};
use crate::errors::app_error::AppError;
use crate::models::auth::users::{MidFieldsUser, MinFieldsUser};
use crate::services::auth::user_management::IUserManagementService;
use crate::state::app_state::AppState;

#[get("/")]
async fn get_users(state: web::Data<Arc<AppState>>) -> Result<HttpResponse, AppError> {
    let users = state.get_services().auth_services.user_management_service.get_users().await?;

    if users.is_empty() {
        return Err(AppError::NoContent("There are no users".to_string()));
    }

    let api_resp: ApiResponse<Vec<MinFieldsUser>> = ApiResponse {
        status: "success".to_string(),
        message: "Users fetched successfully".to_string(),
        data: Some(users),
    };

    Ok(HttpResponse::Ok().json(api_resp))
}

#[get("/{id}")]
async fn get_user(state: web::Data<Arc<AppState>>, path: web::Path<(String,)>) -> Result<HttpResponse, AppError> {
    let user = state.get_services().auth_services.user_management_service.get_user(&path.0).await?
        .ok_or_else(|| AppError::NotFound(format!("User with id {} not found", &path.0)))?;

    let api_resp: ApiResponse<MidFieldsUser> = ApiResponse {
        status: "success".to_string(),
        message: "User fetched successfully".to_string(),
        data: Some(user),
    };

    Ok(HttpResponse::Ok().json(api_resp))
}


#[delete("/me")]
pub async fn delete_user(state: web::Data<Arc<AppState>>, req: web::Json<DeleteUserReq>) -> Result<HttpResponse, AppError> {
    state.get_services().auth_services.user_management_service.delete_user(&state, req.0).await?;

    let api_resp: ApiResponse<String> = ApiResponse {
        status: "success".to_string(),
        message: "User deleted successfully!".to_string(),
        data: None,
    };

    Ok(HttpResponse::Ok().json(api_resp))
}


#[put("/me")]
pub async fn update_user(state: web::Data<Arc<AppState>>, req: web::Json<UpdateUserReq>) -> Result<HttpResponse, AppError> {
    let res = state.get_services().auth_services.user_management_service.update_user(&state, &req.0).await?;

    let api_resp: ApiResponse<MidFieldsUser> = ApiResponse {
        status: "success".to_string(),
        message: "User updated successfully!".to_string(),
        data: Some(res),
    };

    Ok(HttpResponse::Ok().json(api_resp))
}


pub fn user_management_service(cfg: &mut web::ServiceConfig) {
    cfg
        .service(get_users)
        .service(get_user)
        .service(delete_user)
        .service(update_user)
    ;
}