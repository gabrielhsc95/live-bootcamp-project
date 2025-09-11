use std::sync::Arc;
use tokio::sync::RwLock;

use auth_service::Application;
use auth_service::app_state::AppState;
use auth_service::services::hashmap_two_fa_code_store::HashMapTwoFACodeStore;
use auth_service::services::hashmap_user_store::HashmapUserStore;
use auth_service::services::hashset_banned_token_store::HashSetBannedTokenStore;
use auth_service::services::mock_mail_client::MockEmailClient;
use auth_service::utils::constants::prod::APP_ADDRESS;

#[tokio::main]
async fn main() {
    let user_store = HashmapUserStore::default();
    let user_store = Arc::new(RwLock::new(user_store));
    let banned_token_store = HashSetBannedTokenStore::default();
    let banned_token_store = Arc::new(RwLock::new(banned_token_store));
    let two_fa_code_store = HashMapTwoFACodeStore::default();
    let two_fa_code_store = Arc::new(RwLock::new(two_fa_code_store));
    let email_client = MockEmailClient::default();
    let email_client = Arc::new(RwLock::new(email_client));
    let app_state = AppState::new(
        user_store,
        banned_token_store,
        two_fa_code_store,
        email_client,
    );

    let app = Application::build(app_state, APP_ADDRESS)
        .await
        .expect("Failed to build app");
    app.run().await.expect("Failed to run app");
}
