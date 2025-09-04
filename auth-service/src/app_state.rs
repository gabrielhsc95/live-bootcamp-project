use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::{BannedTokenStore, UserStore};

#[derive(Clone)]
pub struct AppState<T: UserStore, U: BannedTokenStore> {
    pub user_store: Arc<RwLock<T>>,
    pub banned_token_store: Arc<RwLock<U>>,
}

impl<T: UserStore, U: BannedTokenStore> AppState<T, U> {
    pub fn new(user_store: Arc<RwLock<T>>, banned_token_store: Arc<RwLock<U>>) -> Self {
        Self {
            user_store,
            banned_token_store,
        }
    }
}
