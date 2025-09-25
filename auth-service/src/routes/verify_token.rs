use crate::app_state::AppState;
use crate::domain::EmailClient;
use crate::domain::data_stores::{BannedTokenStore, TwoFACodeStore, UserStore};
use crate::domain::error::AuthAPIError;
use crate::utils::auth::validate_token;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}

#[tracing::instrument(name = "Verify Token", skip_all)]
pub async fn verify_token<T: UserStore, U: BannedTokenStore, V: TwoFACodeStore, W: EmailClient>(
    State(state): State<AppState<T, U, V, W>>,
    Json(request): Json<VerifyTokenRequest>,
) -> impl IntoResponse {
    let verification = state
        .banned_token_store
        .read()
        .await
        .contains_token(&request.token)
        .await;
    match verification {
        Ok(true) => return AuthAPIError::InvalidToken.into_response(),
        Ok(false) => {}
        Err(e) => return AuthAPIError::UnexpectedError(e.into()).into_response(),
    }
    match validate_token(&request.token).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => AuthAPIError::InvalidToken.into_response(),
    }
}
