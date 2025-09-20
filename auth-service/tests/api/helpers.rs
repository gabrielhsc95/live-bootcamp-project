use auth_service::Application;
use auth_service::app_state::AppState;
// use auth_service::services::data_stores::hashmap_two_fa_code_store::HashMapTwoFACodeStore;
use auth_service::services::data_stores::postgres_user_store::PostgresUserStore;
use auth_service::services::data_stores::redis_two_fa_code_store::RedisTwoFACodeStore;
// use auth_service::services::data_stores::hashmap_user_store::HashmapUserStore;
use auth_service::get_postgres_pool;
use auth_service::get_redis_client;
use auth_service::utils::constants::REDIS_HOST_NAME;
// use auth_service::services::data_stores::hashset_banned_token_store::HashSetBannedTokenStore;
use auth_service::services::data_stores::redis_banned_token_store::RedisBannedTokenStore;
use auth_service::services::mock_mail_client::MockEmailClient;
use auth_service::utils::constants::DATABASE_URL;
use auth_service::utils::constants::test::APP_ADDRESS;
use reqwest::cookie::Jar;
use sqlx::Connection;
use sqlx::Executor;
use sqlx::PgConnection;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

async fn configure_database(db_conn_string: &str, db_name: &str) {
    let connection = PgPoolOptions::new()
        .connect(db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to create database.");

    // Connect to new database
    let db_conn_string = format!("{}/{}", db_conn_string, db_name);

    let connection = PgPoolOptions::new()
        .connect(&db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Run migrations against new database
    sqlx::migrate!()
        .run(&connection)
        .await
        .expect("Failed to migrate the database");
}

async fn configure_postgresql() -> (String, PgPool) {
    let postgresql_conn_url = DATABASE_URL.to_owned();

    let db_name = Uuid::new_v4().to_string();

    configure_database(&postgresql_conn_url, &db_name).await;

    let postgresql_conn_url_with_db = format!("{}/{}", postgresql_conn_url, db_name);

    // Create a new connection pool and return it
    (
        db_name,
        get_postgres_pool(&postgresql_conn_url_with_db)
            .await
            .expect("Failed to create Postgres connection pool!"),
    )
}

async fn delete_database(db_name: &str) {
    let postgresql_conn_url: String = DATABASE_URL.to_owned();

    let mut connection = PgConnection::connect(&postgresql_conn_url)
        .await
        .expect("Failed to connect to Postgres");

    // Kill any active connections to the database
    connection
        .execute(
            format!(
                r#"
                SELECT pg_terminate_backend(pg_stat_activity.pid)
                FROM pg_stat_activity
                WHERE pg_stat_activity.datname = '{}'
                  AND pid <> pg_backend_pid();
        "#,
                db_name
            )
            .as_str(),
        )
        .await
        .expect("Failed to drop the database.");

    // Drop the database
    connection
        .execute(format!(r#"DROP DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to drop the database.");
}

fn configure_redis() -> redis::Connection {
    get_redis_client(REDIS_HOST_NAME.to_owned())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}

pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub http_client: reqwest::Client,
    pub banned_token_store: Arc<RwLock<RedisBannedTokenStore>>,
    pub two_fa_code_store: Arc<RwLock<RedisTwoFACodeStore>>,
    db_name: String,
    clean_up_called: bool,
}

impl TestApp {
    pub async fn new() -> Self {
        let (db_name, pg_pool) = configure_postgresql().await;
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
            // this is because we need access at testing, and it also goes to Self
            banned_token_store.clone(),
            // this is because we need access at testing, and it also goes to Self
            two_fa_code_store.clone(),
            email_client,
        );
        let app = Application::build(app_state, APP_ADDRESS)
            .await
            .expect("Failed to build app");
        let address = format!("http://{}", app.address.clone());
        let cookie_jar = Arc::new(Jar::default());
        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());
        let http_client = reqwest::Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .unwrap();

        Self {
            address,
            cookie_jar,
            http_client,
            banned_token_store: banned_token_store.clone(),
            two_fa_code_store: two_fa_code_store.clone(),
            db_name,
            clean_up_called: false,
        }
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(format!("{}/", self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_signup<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/signup", self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/login", self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_2fa<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/verify-2fa", self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_logout(&self) -> reqwest::Response {
        self.http_client
            .post(format!("{}/logout", self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_token<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/verify-token", self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn clean_up(&mut self) {
        delete_database(&self.db_name).await;
        self.clean_up_called = true;
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        if !self.clean_up_called {
            panic!("The test didn't called clean_up()");
        }
    }
}

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}
