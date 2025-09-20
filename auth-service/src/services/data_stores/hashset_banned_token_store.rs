use crate::domain::data_stores::{BannedTokenStore, BannedTokenStoreError};
use std::collections::HashSet;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Default)]
pub struct HashSetBannedTokenStore {
    pub tokens: HashSet<String>,
}

#[async_trait::async_trait]
impl BannedTokenStore for HashSetBannedTokenStore {
    async fn add_token(&mut self, token: String) -> Result<(), BannedTokenStoreError> {
        self.tokens.insert(token);
        Ok(())
    }
    async fn contains_token(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        let result = self.tokens.contains(token);
        Ok(result)
    }
}
