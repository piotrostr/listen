// Generic retry function
pub async fn retry_with_backoff<F, Fut, T, E>(operation_name: &str, f: F) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    const MAX_RETRIES: u32 = 4;

    for attempt in 0..=MAX_RETRIES {
        if attempt > 0 {
            // Exponential backoff: 100ms, 400ms, 900ms
            let backoff_ms = 100 * attempt * attempt;
            tracing::warn!(
                "Retrying {} (attempt {}/{}), waiting {}ms",
                operation_name,
                attempt,
                MAX_RETRIES,
                backoff_ms
            );
            tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms as u64)).await;
        }

        match f().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                // On last attempt, return the error
                if attempt == MAX_RETRIES {
                    tracing::error!(
                        error = ?err,
                        "All {} attempts failed after {} retries",
                        operation_name,
                        MAX_RETRIES
                    );
                    return Err(err);
                }

                tracing::warn!(
                    error = ?err,
                    attempt = attempt,
                    max_retries = MAX_RETRIES,
                    "{} attempt failed",
                    operation_name
                );
            }
        }
    }

    unreachable!("Loop should either return Ok result or final Err")
}
