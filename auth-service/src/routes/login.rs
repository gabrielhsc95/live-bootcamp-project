use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
pub async fn login(Json(request): Json<LoginRequest>) -> impl IntoResponse {
    StatusCode::OK.into_response()
}
