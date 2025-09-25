use crate::app_state::AppState;
use crate::domain::EmailClient;
use crate::domain::data_stores::{BannedTokenStore, TwoFACodeStore, UserStore};
use crate::domain::error::AuthAPIError;
use crate::utils::auth::validate_token;
use crate::utils::constants::JWT_COOKIE_NAME;
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::cookie::{Cookie, CookieJar};

#[tracing::instrument(name = "Logout", skip_all)]
pub async fn logout<T: UserStore, U: BannedTokenStore, V: TwoFACodeStore, W: EmailClient>(
    State(state): State<AppState<T, U, V, W>>,
    jar: CookieJar,
) -> (CookieJar, impl IntoResponse) {
    let cookie = match jar.get(JWT_COOKIE_NAME) {
        Some(cookie) => cookie,
        None => return (jar, AuthAPIError::MissingToken.into_response()),
    };
    let token = cookie.value().to_owned();
    let _ = match validate_token(&token).await {
        Ok(_) => {
            match state
                .banned_token_store
                .write()
                .await
                .add_token(token)
                .await
            {
                Ok(_) => {}
                Err(e) => return (jar, AuthAPIError::UnexpectedError(e.into()).into_response()),
            }
        }
        Err(_) => return (jar, AuthAPIError::InvalidToken.into_response()),
    };

    let cookie_to_remove = Cookie::build((JWT_COOKIE_NAME, ""))
        .path("/")
        .http_only(true);

    let updated_jar = jar.remove(cookie_to_remove);
    (updated_jar, StatusCode::OK.into_response())
}
