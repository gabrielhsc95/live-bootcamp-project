use crate::helpers::TestApp;
use crate::helpers::get_random_email;
use auth_service::domain::{Email, TwoFACodeStore};
use auth_service::domain::{LoginAttemptId, TwoFACode};
use auth_service::utils::constants::JWT_COOKIE_NAME;

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let mut app = TestApp::new().await;
    let random_email = get_random_email();
    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();

    let test_cases = [
        serde_json::json!({
            "loginAttemptId": login_attempt_id.as_ref().to_owned(),
            "2FACode": two_fa_code.as_ref().to_owned(),
        }),
        serde_json::json!({
            "email": random_email,
            "2FACode": two_fa_code.as_ref().to_owned(),
        }),
        serde_json::json!({
            "email": random_email,
            "loginAttemptId": login_attempt_id.as_ref().to_owned(),
        }),
    ];
    for tc in test_cases {
        let response = app.post_verify_2fa(&tc).await;
        assert_eq!(response.status(), 422, "Failed for input: {:?}", tc);
    }
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let mut app = TestApp::new().await;
    let random_email = get_random_email();
    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();

    let test_cases = [
        serde_json::json!({
            "email": "wrong.email.com".to_owned(),
            "loginAttemptId": login_attempt_id.as_ref().to_owned(),
            "2FACode": two_fa_code.as_ref().to_owned(),
        }),
        serde_json::json!({
            "email": random_email,
            "loginAttemptId": "not_an_uuid_4".to_owned(),
            "2FACode": two_fa_code.as_ref().to_owned(),
        }),
        serde_json::json!({
            "email": random_email,
            "loginAttemptId": login_attempt_id.as_ref().to_owned(),
            "2FACode": "not_2fa_code".to_owned(),
        }),
    ];

    for tc in test_cases {
        let response = app.post_verify_2fa(&tc).await;
        assert_eq!(response.status(), 400, "Failed for input: {:?}", tc);
    }
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let mut app = TestApp::new().await;
    // there is a near zero chance of correct and wrong be the same
    // but I am accepting the odds for this test.
    let random_email = get_random_email();

    let body = serde_json::json!({
        "email": random_email,
        "password": "Password1!",
        "requires2FA": true
    });
    app.post_signup(&body).await;

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "Password1!",
    });

    // wrong, but valid email
    app.post_login(&login_body).await;
    let (login_attempt_id, two_fa_code) = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(&random_email).unwrap())
        .await
        .unwrap();

    let test_case = serde_json::json!({
        "email": "wrong@email.com",
        "loginAttemptId": login_attempt_id.as_ref(),
        "2FACode": two_fa_code.as_ref(),
    });
    let response = app.post_verify_2fa(&test_case).await;
    assert_eq!(response.status(), 401, "Failed for input: {:?}", test_case);

    // wrong, but valid loginAttemptId
    app.post_login(&login_body).await;
    let (_, two_fa_code) = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(&random_email).unwrap())
        .await
        .unwrap();

    let wrong_login_attempt_id = LoginAttemptId::default();
    let test_case = serde_json::json!({
        "email": random_email,
        "loginAttemptId": wrong_login_attempt_id.as_ref(),
        "2FACode": two_fa_code.as_ref(),
    });
    let response = app.post_verify_2fa(&test_case).await;
    assert_eq!(response.status(), 401, "Failed for input: {:?}", test_case);

    // wrong, but valid 2FACode
    app.post_login(&login_body).await;
    let (login_attempt_id, _) = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(&random_email).unwrap())
        .await
        .unwrap();

    let wrong_two_fa_code = TwoFACode::default();
    let test_case = serde_json::json!({
        "email": "wrong@email.com",
        "loginAttemptId": login_attempt_id.as_ref(),
        "2FACode": wrong_two_fa_code.as_ref(),
    });
    let response = app.post_verify_2fa(&test_case).await;
    assert_eq!(response.status(), 401, "Failed for input: {:?}", test_case);
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_old_code() {
    let mut app = TestApp::new().await;
    let signup_body = serde_json::json!({
        "email": "valid@email.com",
        "password": "Password1!",
        "requires2FA": true
    });
    app.post_signup(&signup_body).await;
    let login_body = serde_json::json!({
        "email": "valid@email.com",
        "password": "Password1!",
    });
    app.post_login(&login_body).await;
    let old_2fa_code = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse("valid@email.com").unwrap())
        .await
        .unwrap()
        .1;
    app.post_login(&login_body).await;
    let login_attempt_id = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse("valid@email.com").unwrap())
        .await
        .unwrap()
        .0;
    let verify_2fa_body = serde_json::json!({
        "email": "valid@email.com",
        "loginAttemptId": login_attempt_id.as_ref(),
        "2FACode": old_2fa_code.as_ref(),
    });
    let response = app.post_verify_2fa(&verify_2fa_body).await;
    assert_eq!(response.status(), 401);
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_same_code_twice() {
    let mut app = TestApp::new().await;
    let signup_body = serde_json::json!({
        "email": "valid@email.com",
        "password": "Password1!",
        "requires2FA": true
    });
    app.post_signup(&signup_body).await;
    let login_body = serde_json::json!({
        "email": "valid@email.com",
        "password": "Password1!",
    });
    app.post_login(&login_body).await;
    let (login_attempt_id, two_fa_code) = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse("valid@email.com").unwrap())
        .await
        .unwrap();
    let verify_2fa_body = serde_json::json!({
        "email": "valid@email.com",
        "loginAttemptId": login_attempt_id.as_ref(),
        "2FACode": two_fa_code.as_ref(),
    });
    app.post_verify_2fa(&verify_2fa_body).await;
    let response = app.post_verify_2fa(&verify_2fa_body).await;
    assert_eq!(response.status(), 401);
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_200_if_correct_code() {
    let mut app = TestApp::new().await;
    let signup_body = serde_json::json!({
        "email": "valid@email.com",
        "password": "Password1!",
        "requires2FA": true
    });
    app.post_signup(&signup_body).await;
    let login_body = serde_json::json!({
        "email": "valid@email.com",
        "password": "Password1!",
    });
    app.post_login(&login_body).await;
    let (login_attempt_id, two_fa_code) = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse("valid@email.com").unwrap())
        .await
        .unwrap();

    let test_case = serde_json::json!({
        "email": "valid@email.com",
        "loginAttemptId": login_attempt_id.as_ref(),
        "2FACode": two_fa_code.as_ref(),
    });
    let response = app.post_verify_2fa(&test_case).await;

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");
    // note that is ! (not) is_empty
    assert!(!auth_cookie.value().is_empty());
    app.clean_up().await;
}
