use std::collections::HashMap;

use crate::domain::{
    data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError},
    email::Email,
};

#[cfg(test)]
mod tests;

#[derive(Default, Clone, Debug)]
pub struct HashMapTwoFACodeStore {
    codes: HashMap<Email, (LoginAttemptId, TwoFACode)>,
}

#[async_trait::async_trait]
impl TwoFACodeStore for HashMapTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        let value = self.codes.insert(email, (login_attempt_id, code));
        if value.is_some() {
            return Err(TwoFACodeStoreError::UnexpectedError);
        }
        Ok(())
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        let value = self.codes.remove(email);
        if value.is_none() {
            return Err(TwoFACodeStoreError::UnexpectedError);
        }
        Ok(())
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        match self.codes.get(email) {
            Some(value) => Ok(value.clone()),
            None => Err(TwoFACodeStoreError::UnexpectedError),
        }
    }
}
