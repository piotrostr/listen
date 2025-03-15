use crate::engine::pipeline::Notification;
use anyhow::Result;
use privy::Privy;
use resend_rs::types::CreateEmailBaseOptions;
use resend_rs::Resend;
use std::sync::Arc;

pub async fn send_notification(
    privy: Arc<Privy>,
    user_id: &str,
    notification: &Notification,
) -> Result<()> {
    let recipient_email = privy.get_email_by_user_id(user_id).await?;
    let api_key = std::env::var("EMAIL_API_KEY")?;
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

    Ok(())
}
