use crate::helpers::TestApp;

use crate::helpers::get_random_email;

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;

    let random_email = get_random_email();
    let test_cases = [
        serde_json::json!({
            "password": "password",
            "requires2FA": true
        }),
        serde_json::json!({
            "email": random_email,
            "password": "password"
        }),
        serde_json::json!({
            "email": random_email,
            "requires2FA": true
        }),
    ];

    for tc in test_cases {
        let response = app.post_signup(&tc).await;
        assert_eq!(response.status(), 422, "Failed for input: {:?}", tc);
    }
}
