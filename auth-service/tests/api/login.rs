use crate::helpers::TestApp;
use crate::helpers::get_random_email;
use auth_service::domain::Email;
use auth_service::domain::TwoFACodeStore;
use auth_service::domain::error::ErrorResponse;
use auth_service::routes::TwoFactorAuthResponse;
use auth_service::utils::constants::JWT_COOKIE_NAME;

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;
    let random_email = get_random_email();
    let test_cases = [
        serde_json::json!({
            "email": random_email,
        }),
        serde_json::json!({
            "password": "password"
        }),
    ];
    for tc in test_cases {
        let response = app.post_login(&tc).await;
        assert_eq!(response.status(), 422, "Failed for input: {:?}", tc);
    }
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;

    let test_cases = [
        // email: no @
        serde_json::json!({
            "email": "wrong.email.com",
            "password": "Password1!",
            "requires2FA": true
        }),
        // email: 2 @
        serde_json::json!({
            "email": "two@at@symbols",
            "password": "Password1!",
            "requires2FA": true
        }),
        // password: less than 8 characters
        serde_json::json!({
            "email": get_random_email(),
            "password": "Small!",
            "requires2FA": true
        }),
        // password: no special character
        serde_json::json!({
            "email": get_random_email(),
            "password": "NoSpecialCharacters",
            "requires2FA": true
        }),
        // password: no number
        serde_json::json!({
            "email": get_random_email(),
            "password": "NoNumber!",
            "requires2FA": true
        }),
        // password: no capital letter
        serde_json::json!({
            "email": get_random_email(),
            "password": "no_capital_letter",
            "requires2FA": true
        }),
    ];
    for tc in test_cases {
        let response = app.post_signup(&tc).await;
        assert_eq!(response.status(), 400, "Failed for input: {:?}", tc);
        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response body to ErrorResponse")
                .error,
            "Invalid credentials".to_owned()
        )
    }
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let app = TestApp::new().await;
    let body = serde_json::json!({
        "email": "valid@email.com",
        "password": "Password1!",
        "requires2FA": true
    });
    let _ = app.post_signup(&body).await;
    let test_cases = [
        // wrong, but valid email
        serde_json::json!({
            "email": "wrong@email.com",
            "password": "Password1!",
            "requires2FA": true
        }),
        // wrong, but valid password
        serde_json::json!({
            "email": "valid@email.com",
            "password": "ValidButWrongPassword!",
            "requires2FA": true
        }),
    ];
    for tc in test_cases {
        let response = app.post_login(&tc).await;
        assert_eq!(response.status(), 401, "Failed for input: {:?}", tc);
        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response body to ErrorResponse")
                .error,
            "Authentication failed".to_owned()
        )
    }
}

#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "Password1!",
        "requires2FA": false
    });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "Password1!",
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");
    // note that is ! (not) is_empty
    assert!(!auth_cookie.value().is_empty());
}

#[tokio::test]
async fn should_return_206_if_valid_credentials_and_2fa_enabled() {
    let app = TestApp::new().await;

    let random_email = get_random_email();
    let email = Email::parse(&random_email).unwrap();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "Password1!",
        "requires2FA": true
    });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "Password1!",
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 206);

    let response_body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");

    assert_eq!(response_body.message, "2FA required".to_owned());

    let login_attempt_id_in_response = response_body.login_attempt_id;

    let (login_attempt_id_in_app, _) = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&email)
        .await
        .unwrap();
    assert_eq!(
        login_attempt_id_in_app.as_ref(),
        login_attempt_id_in_response
    );
}
