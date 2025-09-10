use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::{BannedTokenStore, TwoFACodeStore, UserStore};

#[derive(Clone)]
pub struct AppState<T: UserStore, U: BannedTokenStore, V: TwoFACodeStore> {
    pub user_store: Arc<RwLock<T>>,
    pub banned_token_store: Arc<RwLock<U>>,
    pub two_fa_code_store: Arc<RwLock<V>>,
}

impl<T: UserStore, U: BannedTokenStore, V: TwoFACodeStore> AppState<T, U, V> {
    pub fn new(
        user_store: Arc<RwLock<T>>,
        banned_token_store: Arc<RwLock<U>>,
        two_fa_code_store: Arc<RwLock<V>>,
    ) -> Self {
        Self {
            user_store,
            banned_token_store,
            two_fa_code_store,
        }
    }
}
