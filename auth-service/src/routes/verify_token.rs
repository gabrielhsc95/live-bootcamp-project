use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}

pub async fn verify_token(Json(request): Json<VerifyTokenRequest>) -> impl IntoResponse {
    StatusCode::OK.into_response()
}
