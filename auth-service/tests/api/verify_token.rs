use crate::helpers::TestApp;

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;
    let tc = serde_json::json!({});
    let response = app.post_verify_token(&tc).await;
    assert_eq!(response.status(), 422, "Failed for input: {:?}", tc);
}
