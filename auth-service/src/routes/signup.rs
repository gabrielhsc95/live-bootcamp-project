use crate::domain::data_stores::UserStore;
use crate::domain::{AuthAPIError, BannedTokenStore, EmailClient, TwoFACodeStore};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::domain::User;
use crate::domain::UserStoreError;

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct SignupResponse {
    pub message: String,
}

#[tracing::instrument(name = "Signup", skip_all)]
pub async fn signup<T: UserStore, U: BannedTokenStore, V: TwoFACodeStore, W: EmailClient>(
    State(state): State<AppState<T, U, V, W>>,
    Json(request): Json<SignupRequest>,
) -> impl IntoResponse {
    let user = match User::parse(request.email, request.password, request.requires_2fa) {
        Ok(user) => user,
        Err(_) => {
            return AuthAPIError::InvalidCredentials.into_response();
        }
    };
    let mut user_store = state.user_store.write().await;
    match user_store.add_user(user).await {
        Ok(_) => {
            let response = Json(SignupResponse {
                message: "User created successfully!".to_owned(),
            });
            (StatusCode::CREATED, response).into_response()
        }
        Err(e) => match e {
            UserStoreError::UserAlreadyExists => AuthAPIError::UserAlreadyExists.into_response(),
            UserStoreError::InvalidCredentials => AuthAPIError::InvalidCredentials.into_response(),
            UserStoreError::UnexpectedError(e) => AuthAPIError::UnexpectedError(e).into_response(),
            UserStoreError::UserNotFound => {
                panic!("user not found should not happen inside add_user")
            }
        },
    }
}
