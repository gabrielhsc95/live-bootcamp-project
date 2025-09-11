use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::{BannedTokenStore, EmailClient, TwoFACodeStore, UserStore};

#[derive(Clone)]
pub struct AppState<T: UserStore, U: BannedTokenStore, V: TwoFACodeStore, W: EmailClient> {
    pub user_store: Arc<RwLock<T>>,
    pub banned_token_store: Arc<RwLock<U>>,
    pub two_fa_code_store: Arc<RwLock<V>>,
    pub email_client: Arc<RwLock<W>>,
}

impl<T: UserStore, U: BannedTokenStore, V: TwoFACodeStore, W: EmailClient> AppState<T, U, V, W> {
    pub fn new(
        user_store: Arc<RwLock<T>>,
        banned_token_store: Arc<RwLock<U>>,
        two_fa_code_store: Arc<RwLock<V>>,
        email_client: Arc<RwLock<W>>,
    ) -> Self {
        Self {
            user_store,
            banned_token_store,
            two_fa_code_store,
            email_client,
        }
    }
}
