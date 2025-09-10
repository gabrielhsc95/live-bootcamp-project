use crate::app_state::AppState;
use crate::domain::data_stores::{BannedTokenStore, TwoFACodeStore, UserStore};
use crate::domain::error::AuthAPIError;
use crate::utils::auth::validate_token;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}

pub async fn verify_token<T: UserStore, U: BannedTokenStore, V: TwoFACodeStore>(
    State(state): State<AppState<T, U, V>>,
    Json(request): Json<VerifyTokenRequest>,
) -> impl IntoResponse {
    if !state
        .banned_token_store
        .read()
        .await
        .is_valid(&request.token)
        .await
    {
        return AuthAPIError::InvalidToken.into_response();
    }
    match validate_token(&request.token).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => AuthAPIError::InvalidToken.into_response(),
    }
}
