use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::UserStore;

#[derive(Clone)]
pub struct AppState<T: UserStore> {
    pub user_store: Arc<RwLock<T>>,
}

impl<T: UserStore> AppState<T> {
    pub fn new(user_store: T) -> Self {
        let user_store = Arc::new(RwLock::new(user_store));
        Self { user_store }
    }
}
