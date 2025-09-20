use std::sync::Arc;

use redis::{Commands, Connection};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::domain::{
    Email,
    data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError},
};

#[derive(Serialize, Deserialize)]
struct TwoFATuple(pub String, pub String);

const TEN_MINUTES_IN_SECONDS: u64 = 600;
const TWO_FA_CODE_PREFIX: &str = "two_fa_code:";

fn get_key(email: &Email) -> String {
    format!("{}{}", TWO_FA_CODE_PREFIX, email.as_ref())
}

#[derive(Clone)]
pub struct RedisTwoFACodeStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisTwoFACodeStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl TwoFACodeStore for RedisTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        let key = get_key(&email);
        let mut two_fa_store = self.conn.write().await;
        let two_fa_tuple = TwoFATuple(
            login_attempt_id.as_ref().to_string(),
            code.as_ref().to_string(),
        );
        let two_fa_tuple = match serde_json::to_string(&two_fa_tuple) {
            Ok(two_fa_tuple) => two_fa_tuple,
            Err(_) => return Err(TwoFACodeStoreError::UnexpectedError),
        };
        let setting_result: Result<(), redis::RedisError> =
            two_fa_store.set_ex(key, two_fa_tuple, TEN_MINUTES_IN_SECONDS);
        match setting_result {
            Ok(_) => Ok(()),
            Err(_) => Err(TwoFACodeStoreError::UnexpectedError),
        }
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        let key = get_key(email);
        let mut two_fa_store = self.conn.write().await;
        let del_result: Result<(), redis::RedisError> = two_fa_store.del(key);
        match del_result {
            Ok(_) => Ok(()),
            Err(_) => Err(TwoFACodeStoreError::UnexpectedError),
        }
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        let key = get_key(&email);
        let mut two_fa_store = self.conn.write().await;
        let get_result: Result<String, redis::RedisError> = two_fa_store.get(key);
        match get_result {
            Ok(two_fa_tuple) => {
                let two_fa_tuple = serde_json::from_str::<TwoFATuple>(&two_fa_tuple);
                match two_fa_tuple {
                    Ok(two_fa_tuple) => {
                        let login_attempt_id = LoginAttemptId::new_no_validation(two_fa_tuple.0);
                        let code = TwoFACode::new_no_validation(two_fa_tuple.1);
                        Ok((login_attempt_id, code))
                    }
                    Err(_) => Err(TwoFACodeStoreError::UnexpectedError),
                }
            }
            Err(_) => Err(TwoFACodeStoreError::UnexpectedError),
        }
    }
}
