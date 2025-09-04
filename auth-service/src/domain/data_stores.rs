#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}

use std::collections::HashSet;

use super::User;

#[async_trait::async_trait]
pub trait UserStore: Send + Sync + Clone {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;

    async fn get_user(&self, email: &str) -> Result<&User, UserStoreError>;

    async fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError>;
}

#[async_trait::async_trait]
pub trait BannedTokenStore: Send + Sync + Clone {
    async fn ban_token(&mut self, token: String);

    async fn is_valid(&self, token: &str) -> bool;

    async fn tokens(&self) -> HashSet<String>;
}
