use std::sync::Arc;
use tokio::sync::RwLock;

use auth_service::Application;
use auth_service::app_state::AppState;
use auth_service::get_postgres_pool;
// use auth_service::services::data_stores::hashmap_two_fa_code_store::HashMapTwoFACodeStore;
use auth_service::services::data_stores::redis_two_fa_code_store::RedisTwoFACodeStore;
// use auth_service::services::data_stores::hashmap_user_store::HashmapUserStore;
use auth_service::get_redis_client;
// use auth_service::services::data_stores::hashset_banned_token_store::HashSetBannedTokenStore;
use auth_service::services::data_stores::postgres_user_store::PostgresUserStore;
use auth_service::services::data_stores::redis_banned_token_store::RedisBannedTokenStore;
use auth_service::services::mock_mail_client::MockEmailClient;
use auth_service::utils::constants::DATABASE_URL;
use auth_service::utils::constants::REDIS_HOST_NAME;
use auth_service::utils::constants::prod::APP_ADDRESS;

use sqlx::PgPool;

async fn configure_postgresql() -> PgPool {
    let url = DATABASE_URL.as_str();
    let pg_pool = get_postgres_pool(url)
        .await
        .expect("Failed to create Postgres connection pool!");

    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("Failed to run migrations");

    pg_pool
}

fn configure_redis() -> redis::Connection {
    get_redis_client(REDIS_HOST_NAME.to_owned())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}

#[tokio::main]
async fn main() {
    let pg_pool = configure_postgresql().await;
    let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
    let redis_conn = configure_redis();
    let redis_conn = Arc::new(RwLock::new(redis_conn));
    let banned_token_store = RedisBannedTokenStore::new(redis_conn.clone());
    let banned_token_store = Arc::new(RwLock::new(banned_token_store));
    let two_fa_code_store = RedisTwoFACodeStore::new(redis_conn.clone());
    let two_fa_code_store = Arc::new(RwLock::new(two_fa_code_store));
    let email_client = MockEmailClient::default();
    let email_client = Arc::new(email_client);
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
