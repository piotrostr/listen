use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::db::ClickhouseDb;
use crate::redis_client::RedisClient;
use crate::redis_subscriber::RedisSubscriber;

/// Wrapper to extract Redis client from any app state
pub struct RedisClientExt(pub Arc<RedisClient>);

/// Wrapper to extract Clickhouse DB from any app state
pub struct ClickhouseExt(pub Arc<ClickhouseDb>);

/// Wrapper to extract Redis subscriber from any app state
pub struct RedisSubscriberExt(pub Arc<RedisSubscriber>);

#[derive(Clone)]
pub struct AppState {
    pub redis_subscriber: Arc<RedisSubscriber>,
    pub redis_client: Arc<RedisClient>,
    pub clickhouse_db: Option<Arc<ClickhouseDb>>,
}

impl FromRequest for RedisClientExt {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let redis_client = req
            .app_data::<web::Data<AppState>>()
            .map(|data| data.redis_client.clone());

        Box::pin(async move {
            match redis_client {
                Some(client) => Ok(RedisClientExt(client)),
                None => Err(actix_web::error::ErrorInternalServerError(
                    "Redis client not found in app state",
                )),
            }
        })
    }
}

impl FromRequest for ClickhouseExt {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let clickhouse_db = req
            .app_data::<web::Data<AppState>>()
            .and_then(|data| data.clickhouse_db.clone());

        Box::pin(async move {
            match clickhouse_db {
                Some(db) => Ok(ClickhouseExt(db)),
                None => Err(actix_web::error::ErrorInternalServerError(
                    "Clickhouse DB not found in app state",
                )),
            }
        })
    }
}

impl FromRequest for RedisSubscriberExt {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let redis_subscriber = req
            .app_data::<web::Data<AppState>>()
            .map(|data| data.redis_subscriber.clone());

        Box::pin(async move {
            match redis_subscriber {
                Some(subscriber) => Ok(RedisSubscriberExt(subscriber)),
                None => Err(actix_web::error::ErrorInternalServerError(
                    "Redis subscriber not found in app state",
                )),
            }
        })
    }
}
