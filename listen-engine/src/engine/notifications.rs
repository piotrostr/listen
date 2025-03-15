use crate::engine::pipeline::Notification;
use crate::redis::rate_limits::RateLimitType;
use crate::Engine;
use anyhow::Result;
use resend_rs::types::CreateEmailBaseOptions;
use resend_rs::Resend;

impl Engine {
    pub async fn send_notification(
        &self,
        user_id: &str,
        notification: &Notification,
    ) -> Result<String> {
        let rate_limit = self
            .redis
            .get_rate_limit(user_id, &RateLimitType::EmailNotifications)
            .await?;
        if rate_limit.remaining == 0 {
            return Err(anyhow::anyhow!("Rate limit exceeded"));
        }

        let recipient_email = self.privy.get_email_by_user_id(user_id).await?;
        let api_key = std::env::var("RESEND_API_KEY")?;
        let resend = Resend::new(&api_key);
        let from = "listen@app.listen-rs.com";
        let to = [recipient_email.as_str()];

        let email = CreateEmailBaseOptions::new(from, to, &notification.message)
            .with_text(&notification.message);

        let result = resend
            .emails
            .send(email)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send email: {}", e))?;

        tracing::info!("Email sent with ID: {:?}", result.id);

        Ok(result.id.to_string())
    }
}
