use std::sync::Arc;

use redis::{Commands, Connection};
use tokio::sync::RwLock;

use crate::{
    domain::data_stores::{BannedTokenStore, BannedTokenStoreError},
    utils::auth::TOKEN_TTL_SECONDS,
};

const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

fn get_key(token: &str) -> String {
    format!("{BANNED_TOKEN_KEY_PREFIX}{token}")
}

#[derive(Clone)]
pub struct RedisBannedTokenStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisBannedTokenStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    async fn add_token(&mut self, token: String) -> Result<(), BannedTokenStoreError> {
        let key = get_key(&token);
        let mut banned_token_store = self.conn.write().await;
        let setting_result: Result<(), redis::RedisError> =
            banned_token_store.set_ex(key, true, TOKEN_TTL_SECONDS as u64);
        match setting_result {
            Ok(_) => Ok(()),
            Err(_) => Err(BannedTokenStoreError::UnexpectedError),
        }
    }

    async fn contains_token(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        let mut banned_token_store = self.conn.write().await;
        let key = get_key(token);
        let get_result: Result<bool, redis::RedisError> = banned_token_store.get(key);
        match get_result {
            Ok(result) => Ok(result),
            Err(_) => Err(BannedTokenStoreError::UnexpectedError),
        }
    }
}
