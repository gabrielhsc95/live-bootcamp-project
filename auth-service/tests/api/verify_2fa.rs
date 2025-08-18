use crate::helpers::TestApp;
use crate::helpers::get_random_email;
use uuid::Uuid;

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;
    let random_email = get_random_email();

    let test_cases = [
        serde_json::json!({
            "loginAttemptId": Uuid::new_v4().to_string(),
            "2FACode": Uuid::new_v4().to_string()
        }),
        serde_json::json!({
            "email": random_email,
            "2FACode": Uuid::new_v4().to_string()
        }),
        serde_json::json!({
            "email": random_email,
            "loginAttemptId": Uuid::new_v4().to_string(),
        }),
    ];
    for tc in test_cases {
        let response = app.post_verify_2fa(&tc).await;
        assert_eq!(response.status(), 422, "Failed for input: {:?}", tc);
    }
}
