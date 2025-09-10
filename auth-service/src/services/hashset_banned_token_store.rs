use crate::domain::data_stores::BannedTokenStore;
use std::collections::HashSet;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Default)]
pub struct HashSetBannedTokenStore {
    pub tokens: HashSet<String>,
}

#[async_trait::async_trait]
impl BannedTokenStore for HashSetBannedTokenStore {
    async fn ban_token(&mut self, token: String) {
        self.tokens.insert(token);
    }

    async fn is_valid(&self, token: &str) -> bool {
        !self.tokens.contains(token)
    }

    async fn tokens(&self) -> HashSet<String> {
        self.tokens.clone()
    }
}
