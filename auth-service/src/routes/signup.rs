use crate::domain::AuthAPIError;
use crate::domain::data_stores::UserStore;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::domain::User;

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

pub async fn signup<T: UserStore>(
    State(state): State<AppState<T>>,
    Json(request): Json<SignupRequest>,
) -> impl IntoResponse {
    let user = match User::parse(request.email, request.password, false) {
        Ok(user) => user,
        Err(_) => {
            return AuthAPIError::InvalidCredentials.into_response();
        }
    };
    let mut user_store = state.user_store.write().await;
    if let Ok(_) = user_store.add_user(user).await {
        let response = Json(SignupResponse {
            message: "User created successfully!".to_string(),
        });

        (StatusCode::CREATED, response).into_response()
    } else {
        return AuthAPIError::UserAlreadyExists.into_response();
    }
}
