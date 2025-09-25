use super::Email;
use color_eyre::eyre::Report;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmailClientError {
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}

#[async_trait::async_trait]
pub trait EmailClient: Send + Sync + Clone {
    async fn send_email(
        &self,
        recipient: &Email,
        subject: &str,
        content: &str,
    ) -> Result<(), EmailClientError>;
}
