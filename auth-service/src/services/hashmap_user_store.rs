use crate::domain::data_stores::{UserStore, UserStoreError};
use std::collections::HashMap;

use crate::domain::{Email, User};

#[cfg(test)]
mod tests;

#[derive(Clone)]
pub struct HashmapUserStore {
    users: HashMap<Email, User>,
}

impl Default for HashmapUserStore {
    fn default() -> Self {
        Self {
            users: HashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let email = user.email();

        if self.users.contains_key(&email) {
            return Err(UserStoreError::UserAlreadyExists);
        }
        self.users.insert(email, user);
        Ok(())
    }

    async fn get_user(&self, email: &str) -> Result<&User, UserStoreError> {
        let email = Email {
            email: email.to_string(),
        };
        self.users.get(&email).ok_or(UserStoreError::UserNotFound)
    }

    async fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        let user = self.get_user(email).await?;
        if user.password_str() != password {
            return Err(UserStoreError::InvalidCredentials);
        }
        Ok(())
    }
}
