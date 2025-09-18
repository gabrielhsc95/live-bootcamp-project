use std::error::Error;

use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
    password_hash::{SaltString, rand_core::OsRng},
};

use sqlx::PgPool;

use crate::domain::{
    Email, User,
    data_stores::{UserStore, UserStoreError},
};

async fn verify_password_hash(
    expected_password_hash: String,
    password_candidate: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let argon2 = Argon2::default();
    let verification = tokio::task::spawn_blocking(move || {
        match PasswordHash::new(&expected_password_hash) {
            Ok(hash) => match argon2.verify_password(password_candidate.as_bytes(), &hash) {
                Ok(_) => return Ok(()),
                Err(e) => return Err(Box::new(e) as Box<dyn Error + Send + Sync>),
            },
            Err(e) => return Err(Box::new(e) as Box<dyn Error + Send + Sync>),
        };
    })
    .await?;
    verification
}

async fn compute_password_hash(password: String) -> Result<String, Box<dyn Error + Send + Sync>> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, None)?,
    );
    let hash_result = tokio::task::spawn_blocking(move || {
        match argon2.hash_password(password.as_bytes(), &salt) {
            Ok(hash) => Ok(hash.to_string()),
            Err(e) => Err(Box::new(e) as Box<dyn Error + Send + Sync>),
        }
    })
    .await?;

    hash_result
}

#[derive(Clone)]
pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let email = user.email_str();
        let password = user.password_str();
        let requires_2fa = user.requires_2fa();

        let password_hash = match compute_password_hash(password.to_owned()).await {
            Ok(hash) => hash,
            Err(_) => {
                return Err(UserStoreError::UnexpectedError);
            }
        };
        let query = sqlx::query!(
            "INSERT INTO users (email, password_hash, requires_2fa) VALUES ($1, $2, $3)",
            email,
            password_hash,
            requires_2fa,
        )
        .execute(&self.pool)
        .await;
        match query {
            Ok(_) => Ok(()),
            Err(_) => Err(UserStoreError::UnexpectedError),
        }
    }

    async fn get_user(&self, email: &str) -> Result<User, UserStoreError> {
        let email = match Email::parse(email) {
            Ok(email) => email,
            Err(_) => {
                return Err(UserStoreError::InvalidCredentials);
            }
        };
        let query = sqlx::query!(
            "SELECT email, password_hash, requires_2fa FROM users WHERE email = $1",
            email.as_ref(),
        )
        .fetch_one(&self.pool)
        .await;
        match query {
            Ok(row) => Ok(User::new_no_validation(
                row.email,
                row.password_hash,
                row.requires_2fa,
            )),
            Err(_) => {
                return Err(UserStoreError::UserNotFound);
            }
        }
    }

    async fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        let user = self.get_user(email).await?;
        match verify_password_hash(user.password_str().to_owned(), password.to_owned()).await {
            Ok(_) => Ok(()),
            Err(_) => Err(UserStoreError::InvalidCredentials),
        }
    }
}
