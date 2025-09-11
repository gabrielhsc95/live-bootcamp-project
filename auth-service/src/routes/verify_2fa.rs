use crate::domain::EmailClient;
use crate::domain::data_stores::{BannedTokenStore, TwoFACodeStore, UserStore};
use crate::utils::auth::generate_auth_cookie;
use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, LoginAttemptId, TwoFACode},
};
use axum::{
    Json, body::Body, extract::State, http::StatusCode, response::IntoResponse, response::Response,
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Verify2FARequest {
    pub email: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
    #[serde(rename = "2FACode")]
    pub twofa_code: String,
}
pub async fn verify_2fa<T: UserStore, U: BannedTokenStore, V: TwoFACodeStore, W: EmailClient>(
    State(state): State<AppState<T, U, V, W>>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>,
) -> (CookieJar, Response<Body>) {
    let email = match Email::parse(request.email.as_str()) {
        Ok(email) => email,
        Err(_) => {
            return (jar, AuthAPIError::InvalidCredentials.into_response());
        }
    };
    let login_attempt_id_request =
        match LoginAttemptId::parse(request.login_attempt_id.as_str().to_owned()) {
            Ok(login_attempt_id) => login_attempt_id,
            Err(_) => {
                return (jar, AuthAPIError::InvalidCredentials.into_response());
            }
        };
    let twofa_code_request = match TwoFACode::parse(request.twofa_code.as_str().to_owned()) {
        Ok(twofa_code) => twofa_code,
        Err(_) => {
            return (jar, AuthAPIError::InvalidCredentials.into_response());
        }
    };

    let (login_attempt_id_store, twofa_code_store) =
        match state.two_fa_code_store.read().await.get_code(&email).await {
            Ok(get_code_results) => get_code_results,
            Err(_) => {
                return (jar, AuthAPIError::IncorrectCredentials.into_response());
            }
        };
    if login_attempt_id_request != login_attempt_id_store {
        return (jar, AuthAPIError::IncorrectCredentials.into_response());
    }
    if twofa_code_request != twofa_code_store {
        return (jar, AuthAPIError::IncorrectCredentials.into_response());
    }
    if state
        .two_fa_code_store
        .write()
        .await
        .remove_code(&email)
        .await
        .is_err()
    {
        return (jar, AuthAPIError::UnexpectedError.into_response());
    }

    let auth_cookie = match generate_auth_cookie(&email) {
        Ok(cookie) => cookie,
        Err(_) => return (jar, AuthAPIError::IncorrectCredentials.into_response()),
    };

    let updated_jar = jar.add(auth_cookie);

    (updated_jar, StatusCode::OK.into_response())
}
