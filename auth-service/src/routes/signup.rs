use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

pub async fn signup(Json(request): Json<SignupRequest>) -> impl IntoResponse {
    // TODO
    StatusCode::OK.into_response()
}
