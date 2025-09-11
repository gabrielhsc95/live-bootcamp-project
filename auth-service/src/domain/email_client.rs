use super::Email;

#[async_trait::async_trait]
pub trait EmailClient: Send + Sync + Clone {
    async fn send_email(
        &self,
        recipient: &Email,
        subject: &str,
        content: &str,
    ) -> Result<(), String>;
}
