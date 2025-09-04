use crate::helpers::TestApp;
use auth_service::utils::constants::JWT_COOKIE_NAME;
use reqwest::Url;

#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing() {
    let app = TestApp::new().await;

    let response = app.post_logout().await;
    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let app = TestApp::new().await;

    // add invalid cookie
    app.cookie_jar.add_cookie_str(
        &format!(
            "{}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let response = app.post_logout().await;
    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn should_return_200_if_valid_jwt_cookie() {
    let app = TestApp::new().await;
    let body_signup = serde_json::json!({
        "email": "valid@email.com",
        "password": "Password1!",
        "requires2FA": true
    });
    let _ = app.post_signup(&body_signup).await;
    let body_login = serde_json::json!({
        "email": "valid@email.com",
        "password": "Password1!"
    });
    let response = app.post_login(&body_login).await;
    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    let response = app.post_logout().await;
    assert_eq!(response.status(), 200);
    assert!(
        app.banned_token_store
            .read()
            .await
            .clone()
            .tokens
            .contains(auth_cookie.value())
    );
}

#[tokio::test]
async fn should_return_400_if_logout_called_twice_in_a_row() {
    let app = TestApp::new().await;
    let body_signup = serde_json::json!({
        "email": "valid@email.com",
        "password": "Password1!",
        "requires2FA": true
    });
    let _ = app.post_signup(&body_signup).await;
    let body_login = serde_json::json!({
        "email": "valid@email.com",
        "password": "Password1!"
    });
    let _ = app.post_login(&body_login).await;
    let _ = app.post_logout().await;
    let response = app.post_logout().await;
    assert_eq!(response.status(), 400);
}
