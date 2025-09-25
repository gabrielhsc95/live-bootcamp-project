use crate::app_state::AppState;
use crate::domain::data_stores::{BannedTokenStore, TwoFACodeStore, UserStore};
use crate::domain::{AuthAPIError, Email, EmailClient, LoginAttemptId, TwoFACode};
use crate::utils::auth::generate_auth_cookie;
use axum::{
    Json, body::Body, extract::State, http::StatusCode, response::IntoResponse, response::Response,
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorAuthResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum LoginResponse {
    RegularAuth,
    TwoFactorAuth(TwoFactorAuthResponse),
}

#[tracing::instrument(name = "No 2FA", skip_all)]
async fn handle_no_2fa(email: &Email, jar: CookieJar) -> (CookieJar, Response<Body>) {
    let auth_cookie = match generate_auth_cookie(email) {
        Ok(cookie) => cookie,
        Err(_) => return (jar, AuthAPIError::IncorrectCredentials.into_response()),
    };

    let updated_jar = jar.add(auth_cookie);

    (updated_jar, StatusCode::OK.into_response())
}

#[tracing::instrument(name = "Handle 2FA", skip_all)]
async fn handle_2fa<T: UserStore, U: BannedTokenStore, V: TwoFACodeStore, W: EmailClient>(
    email: &Email,
    state: &AppState<T, U, V, W>,
    jar: CookieJar,
) -> (CookieJar, Response<Body>) {
    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();

    let body = TwoFactorAuthResponse {
        message: "2FA required".to_owned(),
        login_attempt_id: login_attempt_id.as_ref().to_owned(),
    };

    if let Err(e) = state
        .two_fa_code_store
        .write()
        .await
        .add_code(email.clone(), login_attempt_id.clone(), two_fa_code.clone())
        .await
    {
        return (jar, AuthAPIError::UnexpectedError(e.into()).into_response());
    }

    if let Err(e) = state
        .email_client
        .send_email(&email, "2FA code", &two_fa_code.as_ref())
        .await
    {
        return (jar, AuthAPIError::UnexpectedError(e.into()).into_response());
    }

    let body = Json(body);
    let response = (StatusCode::PARTIAL_CONTENT, body).into_response();
    (jar, response)
}

#[tracing::instrument(name = "Login", skip_all)]
pub async fn login<T: UserStore, U: BannedTokenStore, V: TwoFACodeStore, W: EmailClient>(
    State(state): State<AppState<T, U, V, W>>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> (CookieJar, Response<Body>) {
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
    if user.requires_2fa() {
        handle_2fa(&user.email(), &state, jar).await
    } else {
        handle_no_2fa(&user.email(), jar).await
    }
}
