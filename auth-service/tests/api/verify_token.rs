use crate::helpers::TestApp;

#[tokio::test]
async fn verify_token() {
    let app = TestApp::new().await;
    // let map = HashMap::new();
    let response = app.post_verify_token().await;

    assert_eq!(response.status(), 200);
}
