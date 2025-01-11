use mongodb::{
    bson::{Bson, Document},
    options::ClientOptions,
    Client, Collection,
};
use std::error::Error;

use crate::{checker_service::TokenResult, util::env};

pub struct Collector {
    collection: Collection<Document>,
}

pub async fn new() -> Result<Collector, Box<dyn Error>> {
    let client_options = ClientOptions::parse(&env("MONGO_URL")).await?;
    let client = Client::with_options(client_options)?;

    let db = client.database("db");
    let collection = db.collection::<Document>("token_results");

    Ok(Collector { collection })
}

impl Collector {
    pub async fn insert(
        &self,
        token_result: TokenResult,
    ) -> Result<Bson, Box<dyn Error>> {
        let doc = mongodb::bson::to_document(&token_result)?;
        let res = self.collection.insert_one(doc, None).await?;
        Ok(res.inserted_id)
    }

    pub async fn insert_generic(
        &self,
        doc: serde_json::Value,
    ) -> Result<Bson, Box<dyn Error>> {
        let doc = mongodb::bson::to_document(&doc)?;
        let res = self.collection.insert_one(doc, None).await?;
        Ok(res.inserted_id)
    }
}

#[cfg(test)]
mod tests {
    use crate::checker_service::TokenResult;

    #[tokio::test]
    #[ignore = "integration test"]
    async fn test_collector() {
        let collector = super::new().await.unwrap();
        let token_result = TokenResult::default();
        collector.insert(token_result).await.unwrap();
    }
}
