use validator::{Validate, ValidationError, ValidationErrors};

use super::Email;
use super::User;
use rand::prelude::*;
use uuid::Uuid;

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}

#[async_trait::async_trait]
pub trait UserStore: Send + Sync + Clone {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;

    async fn get_user(&self, email: &str) -> Result<User, UserStoreError>;

    async fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError>;
}

#[derive(Debug)]
pub enum BannedTokenStoreError {
    UnexpectedError,
}

#[async_trait::async_trait]
pub trait BannedTokenStore: Send + Sync + Clone {
    async fn add_token(&mut self, token: String) -> Result<(), BannedTokenStoreError>;
    async fn contains_token(&self, token: &str) -> Result<bool, BannedTokenStoreError>;
}

#[derive(Debug, PartialEq)]
pub enum TwoFACodeStoreError {
    LoginAttemptIdNotFound,
    UnexpectedError,
}

fn validate_code(code: &str) -> Result<(), ValidationError> {
    if !code.len() == 6 {
        return Err(ValidationError::new("Code is not 6 digits long"));
    }
    if !code.chars().all(|num| num.is_ascii_digit()) {
        return Err(ValidationError::new(
            "Only ASCII digits are part of the code.",
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Validate)]
pub struct TwoFACode {
    #[validate(custom(function = "validate_code"))]
    code: String,
}

impl TwoFACode {
    pub fn parse(code: String) -> Result<Self, ValidationErrors> {
        let code = Self { code };
        code.validate()?;
        Ok(code)
    }

    pub fn new_no_validation(code: String) -> Self {
        // Use with care, since there is no validation, for example, from
        // parsing a TwoFACode from redis, with the assumption if it is in
        // redis it is valid
        Self { code }
    }
}

impl Default for TwoFACode {
    fn default() -> Self {
        let mut rng = rand::rng();
        let nums: Vec<u32> = (0..=999999).collect();
        let code = nums
            .choose(&mut rng)
            .expect("Something went wrong the the sample in rand.");
        let code = format!("{:06}", code);
        Self { code }
    }
}

impl AsRef<str> for TwoFACode {
    fn as_ref(&self) -> &str {
        self.code.as_str()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoginAttemptId(String);

impl LoginAttemptId {
    pub fn parse(id: String) -> Result<Self, String> {
        match Uuid::parse_str(&id) {
            Ok(uuid) => Ok(Self(uuid.to_string())),
            Err(_) => Err("Invalid UUID format".to_owned()),
        }
    }

    pub fn new_no_validation(id: String) -> Self {
        // Use with care, since there is no validation, for example, from
        // parsing a LoginAttemptId from redis, with the assumption if it is in
        // redis it is valid.
        Self(id)
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        let uuid = Uuid::new_v4();
        Self(uuid.to_string())
    }
}

impl AsRef<str> for LoginAttemptId {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

#[async_trait::async_trait]
pub trait TwoFACodeStore: Send + Sync + Clone {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError>;

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError>;

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError>;
}
