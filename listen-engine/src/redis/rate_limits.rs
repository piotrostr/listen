use crate::redis::client::{RedisClient, RedisClientError};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub enum RateLimitType {
    EmailNotifications,
    ActivePipelines,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserPlan {
    Free,
    Basic,
    Premium,
    Enterprise,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RateLimit {
    pub limit: u32,
    pub remaining: u32,
    pub reset_at: Option<u64>, // Unix timestamp when the limit resets
}

impl RateLimitType {
    pub fn key(&self) -> &str {
        match self {
            RateLimitType::EmailNotifications => "email_notifications",
            RateLimitType::ActivePipelines => "active_pipelines",
        }
    }

    pub fn default_limit(&self, plan: Option<UserPlan>) -> u32 {
        let plan = plan.unwrap_or(UserPlan::Free);

        match self {
            RateLimitType::EmailNotifications => match plan {
                _ => 5,
            },
            RateLimitType::ActivePipelines => match plan {
                _ => 1000,
            },
        }
    }

    pub fn default_window(&self) -> Duration {
        match self {
            RateLimitType::EmailNotifications => Duration::from_secs(24 * 60 * 60), // 24 hours
            RateLimitType::ActivePipelines => Duration::from_secs(0), // No expiry for active pipelines
        }
    }

    pub fn is_blocking(&self) -> bool {
        match self {
            RateLimitType::EmailNotifications => true, // Block when limit reached
            RateLimitType::ActivePipelines => true,    // Block when limit reached
        }
    }
}

impl RedisClient {
    pub async fn get_rate_limit(
        &self,
        user_id: &str,
        limit_type: &RateLimitType,
    ) -> Result<RateLimit, RedisClientError> {
        let key = format!("rate_limit:{}:{}", user_id, limit_type.key());

        // Get user-specific limit or fall back to default
        let limit = self.get_user_limit(user_id, limit_type).await?;

        // Get the current count
        let count: Option<u32> = self.get(&key).await?;

        // Get the TTL to determine when the limit resets
        let ttl: Option<i64> = self.ttl(&key).await?;

        let count = count.unwrap_or(0);
        let remaining = if count > limit { 0 } else { limit - count };

        // Convert TTL to reset timestamp if available
        let reset_at = ttl.and_then(|ttl| {
            if ttl > 0 {
                Some((chrono::Utc::now().timestamp() as u64) + (ttl as u64))
            } else {
                None
            }
        });

        Ok(RateLimit {
            limit,
            remaining,
            reset_at,
        })
    }

    pub async fn increment_rate_limit(
        &self,
        user_id: &str,
        limit_type: &RateLimitType,
    ) -> Result<RateLimit, RedisClientError> {
        let key = format!("rate_limit:{}:{}", user_id, limit_type.key());

        // Increment the counter
        let new_count: u32 = self.incr(&key, 1).await?;

        // If this is the first increment, set the expiry
        if new_count == 1 {
            let window = limit_type.default_window();
            if window.as_secs() > 0 {
                self.expire(&key, window.as_secs() as usize).await?;
            }
        }

        let limit = self.get_user_limit(user_id, limit_type).await?;
        let remaining = if new_count > limit {
            0
        } else {
            limit - new_count
        };

        // Get the TTL to determine when the limit resets
        let ttl: Option<i64> = self.ttl(&key).await?;
        let reset_at = ttl.and_then(|ttl| {
            if ttl > 0 {
                Some((chrono::Utc::now().timestamp() as u64) + (ttl as u64))
            } else {
                None
            }
        });

        Ok(RateLimit {
            limit,
            remaining,
            reset_at,
        })
    }

    pub async fn check_rate_limit(
        &self,
        user_id: &str,
        limit_type: &RateLimitType,
    ) -> Result<bool, RedisClientError> {
        let rate_limit = self.get_rate_limit(user_id, limit_type).await?;
        Ok(rate_limit.remaining > 0)
    }

    pub async fn reset_rate_limit(
        &self,
        user_id: &str,
        limit_type: &RateLimitType,
    ) -> Result<(), RedisClientError> {
        let key = format!("rate_limit:{}:{}", user_id, limit_type.key());
        self.del(&key).await?;
        Ok(())
    }

    pub async fn decrement_rate_limit(
        &self,
        user_id: &str,
        limit_type: &RateLimitType,
    ) -> Result<RateLimit, RedisClientError> {
        let key = format!("rate_limit:{}:{}", user_id, limit_type.key());

        // Get current count
        let count: Option<u32> = self.get(&key).await?;

        if let Some(count) = count {
            if count > 0 {
                // Decrement the counter
                let new_count: u32 = self.incr(&key, u32::MAX - 1 + 1).await?; // Equivalent to -1

                let limit = self.get_user_limit(user_id, limit_type).await?;
                let remaining = if new_count > limit {
                    0
                } else {
                    limit - new_count
                };

                // Get the TTL
                let ttl: Option<i64> = self.ttl(&key).await?;
                let reset_at = ttl.and_then(|ttl| {
                    if ttl > 0 {
                        Some((chrono::Utc::now().timestamp() as u64) + (ttl as u64))
                    } else {
                        None
                    }
                });

                return Ok(RateLimit {
                    limit,
                    remaining,
                    reset_at,
                });
            }
        }

        // If we get here, either the key doesn't exist or count is 0
        Ok(RateLimit {
            limit: self.get_user_limit(user_id, limit_type).await?,
            remaining: self.get_user_limit(user_id, limit_type).await?,
            reset_at: None,
        })
    }

    // Set a custom limit for a specific user and limit type
    pub async fn set_user_limit(
        &self,
        user_id: &str,
        limit_type: &RateLimitType,
        limit: u32,
    ) -> Result<(), RedisClientError> {
        let key = format!("user_limit:{}:{}", user_id, limit_type.key());
        self.set(&key, &limit).await?;
        Ok(())
    }

    // Get the limit for a specific user, falling back to default if not set
    pub async fn get_user_limit(
        &self,
        user_id: &str,
        limit_type: &RateLimitType,
    ) -> Result<u32, RedisClientError> {
        let key = format!("user_limit:{}:{}", user_id, limit_type.key());
        let limit: Option<u32> = self.get(&key).await?;
        Ok(limit.unwrap_or_else(|| limit_type.default_limit(None)))
    }
}
