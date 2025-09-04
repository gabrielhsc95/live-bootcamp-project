use crate::app_state::AppState;
use crate::domain::AuthAPIError;
use crate::domain::data_stores::{BannedTokenStore, UserStore};
use crate::utils::auth::generate_auth_cookie;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
pub async fn login<T: UserStore, U: BannedTokenStore>(
    State(state): State<AppState<T, U>>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> (CookieJar, impl IntoResponse) {
    // requires_2fa is always false because here we are just checking if it is a valid email and password.
    // and the parse method in User does that.
    let user_store = state.user_store.read().await;
    let validation = user_store
        .validate_user(&request.email, &request.password)
        .await;
    if validation.is_err() {
        return (jar, AuthAPIError::IncorrectCredentials.into_response());
    }
    let user = match user_store.get_user(&request.email).await {
        Ok(stored_user) => stored_user,
        Err(_) => {
            // it should happen because user_store.validate_user() already makes sure that
            // the user exists, there should be a user_store.get_user() inside.
            // But even if that's not the case, this is the right error.
            return (jar, AuthAPIError::IncorrectCredentials.into_response());
        }
    };

    let auth_cookie = match generate_auth_cookie(&user.email()) {
        Ok(cookie) => cookie,
        Err(_) => return (jar, AuthAPIError::IncorrectCredentials.into_response()),
    };

    let updated_jar = jar.add(auth_cookie);

    (updated_jar, StatusCode::OK.into_response())
}
