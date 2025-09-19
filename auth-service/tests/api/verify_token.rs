use crate::helpers::TestApp;
use crate::helpers::get_random_email;
use auth_service::utils::constants::JWT_COOKIE_NAME;

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let mut app = TestApp::new().await;
    let tc = serde_json::json!({});
    let response = app.post_verify_token(&tc).await;
    assert_eq!(response.status(), 422, "Failed for input: {:?}", tc);
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_200_valid_token() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();
    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "Password1!",
        "requires2FA": false
    });
    let _ = app.post_signup(&signup_body).await;

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "Password1!",
    });
    let response = app.post_login(&login_body).await;

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    let verify_token_body = serde_json::json!({
        "token": auth_cookie.value()
    });
    let response = app.post_verify_token(&verify_token_body).await;
    assert_eq!(response.status(), 200);
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let mut app = TestApp::new().await;

    let verify_token_body = serde_json::json!({
        "token": "wrong"
    });
    let response = app.post_verify_token(&verify_token_body).await;
    assert_eq!(response.status(), 401);
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_banned_token() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();
    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "Password1!",
        "requires2FA": false
    });
    let _ = app.post_signup(&signup_body).await;

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "Password1!",
    });
    let response = app.post_login(&login_body).await;

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    let _ = app.post_logout().await;

    let verify_token_body = serde_json::json!({
        "token": auth_cookie.value()
    });
    let response = app.post_verify_token(&verify_token_body).await;
    assert_eq!(response.status(), 401);
    app.clean_up().await;
}
