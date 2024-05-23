use dotenv_codegen::dotenv;
use mongodb::{
    bson::{Bson, Document},
    options::ClientOptions,
    Client, Collection,
};
use std::error::Error;

use crate::buyer::TokenResult;

pub struct Collector {
    collection: Collection<Document>,
}

pub async fn new() -> Result<Collector, Box<dyn Error>> {
    let client_options = ClientOptions::parse(dotenv!("MONGO_URL")).await?;
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
}

#[cfg(test)]
mod tests {
    use crate::buyer::TokenResult;

    #[tokio::test]
    async fn test_collector() {
        let collector = super::new().await.unwrap();
        let token_result = TokenResult::default();
        collector.insert(token_result).await.unwrap();
    }
}