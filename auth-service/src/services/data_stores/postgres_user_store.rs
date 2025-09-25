use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
    password_hash::{SaltString, rand_core::OsRng},
};
use color_eyre::eyre::{Context, Result, eyre};

use sqlx::PgPool;

use crate::domain::{
    Email, User,
    data_stores::{UserStore, UserStoreError},
};

#[tracing::instrument(name = "Verify password hash", skip_all)]
async fn verify_password_hash(
    expected_password_hash: String,
    password_candidate: String,
) -> Result<()> {
    let current_span: tracing::Span = tracing::Span::current();
    let argon2 = Argon2::default();
    tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let expected_password_hash: PasswordHash<'_> =
                PasswordHash::new(&expected_password_hash)?;
            argon2
                .verify_password(password_candidate.as_bytes(), &expected_password_hash)
                .wrap_err("failed to verify password hash")
        })
    })
    .await?
}

#[tracing::instrument(name = "Computing password hash", skip_all)]
async fn compute_password_hash(password: String) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, None)?,
    );
    let current_span: tracing::Span = tracing::Span::current();
    tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let hash = argon2
                .hash_password(password.as_bytes(), &salt)?
                .to_string();
            Ok(hash)
        })
    })
    .await?
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
    #[tracing::instrument(name = "Adding user to PostgreSQL", skip_all)]
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let email = user.email_str();
        let password = user.password_str();
        let requires_2fa = user.requires_2fa();

        match self.get_user(email).await {
            Ok(_) => return Err(UserStoreError::UserAlreadyExists),
            Err(UserStoreError::UserNotFound) => {} // do nothing
            Err(e) => return Err(UserStoreError::UnexpectedError(e.into())),
        }

        let password_hash = compute_password_hash(password.to_owned())
            .await
            .map_err(|e| UserStoreError::UnexpectedError(e.into()))?;
        sqlx::query!(
            "INSERT INTO users (email, password_hash, requires_2fa) VALUES ($1, $2, $3)",
            email,
            password_hash,
            requires_2fa,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| UserStoreError::UnexpectedError(e.into()))?;

        Ok(())
    }

    #[tracing::instrument(name = "Retrieving user from PostgreSQL", skip_all)]
    async fn get_user(&self, email: &str) -> Result<User, UserStoreError> {
        let email = Email::parse(email).map_err(|_| UserStoreError::InvalidCredentials)?;
        sqlx::query!(
            "SELECT email, password_hash, requires_2fa FROM users WHERE email = $1",
            email.as_ref(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UserStoreError::UnexpectedError(e.into()))?
        .map(|row| {
            Ok(User::parse(row.email, row.password_hash, row.requires_2fa)
                .map_err(|e| UserStoreError::UnexpectedError(eyre!(e)))?)
        })
        .ok_or(UserStoreError::UserNotFound)?
    }

    #[tracing::instrument(name = "Validating user credentials in PostgreSQL", skip_all)]
    async fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        let user = self.get_user(email).await?;
        match verify_password_hash(user.password_str().to_owned(), password.to_owned()).await {
            Ok(_) => Ok(()),
            Err(_) => Err(UserStoreError::InvalidCredentials),
        }
    }
}
