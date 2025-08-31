use crate::app_state::AppState;
use crate::domain::AuthAPIError;
use crate::domain::data_stores::UserStore;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
pub async fn login<T: UserStore>(
    State(state): State<AppState<T>>,
    Json(request): Json<LoginRequest>,
) -> impl IntoResponse {
    // requires_2fa is always false because here we are just checking if it is a valid email and password.
    // and the parse method in User does that.
    let user_store = state.user_store.read().await;
    let validation = user_store
        .validate_user(&request.email, &request.password)
        .await;
    if validation.is_err() {
        return AuthAPIError::IncorrectCredentials.into_response();
    }
    let _ = match user_store.get_user(&request.email).await {
        Ok(stored_user) => stored_user,
        Err(_) => {
            // it should happen because user_store.validate_user() already makes sure that
            // the user exists, there should be a ser_store.get_user() inside.
            // But even if that's not the case, this is the right error.
            return AuthAPIError::IncorrectCredentials.into_response();
        }
    };

    StatusCode::OK.into_response()
}
