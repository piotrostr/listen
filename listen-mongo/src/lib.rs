use anyhow::{anyhow, Result};
use bson::{doc, Document};
use futures::StreamExt;
use mongodb::{
    options::{ClientOptions, FindOptions},
    Client, Collection, Database,
};
use serde::{de::DeserializeOwned, Serialize};
use std::env;

/// MongoDB client for inserting documents under a given key
pub struct MongoClient {
    client: Client,
    db_name: String,
}

#[derive(Debug, thiserror::Error)]
pub enum MongoError {
    #[error("Failed to insert document: {0}")]
    InsertError(mongodb::error::Error),
    #[error("Failed to find document: {0}")]
    FindError(mongodb::error::Error),
    #[error("Failed to update document: {0}")]
    UpdateError(mongodb::error::Error),
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),
    #[error("Failed to create client: {0}")]
    CreateClientError(mongodb::error::Error),
    #[error("Failed to parse client options: {0}")]
    ParseClientOptionsError(mongodb::error::Error),
    #[error("Failed health check: {0}")]
    HealthCheckError(mongodb::error::Error),
    #[error("Failed to parse UUID: {0}")]
    ParseUuidError(bson::uuid::Error),
    #[error("Failed to convert document to BSON: {0}")]
    ConvertDocumentToBsonError(bson::ser::Error),
    #[error("Failed to get inserted ID: {0}")]
    GetInsertedIdError(anyhow::Error),
}

impl MongoClient {
    /// Creates a new MongoDB client
    pub async fn from_env() -> Result<Self, MongoError> {
        let mongo_uri = env::var("MONGODB_URI")
            .map_err(|_| MongoError::MissingEnvVar("MONGODB_URI".to_string()))?;
        let db_name = env::var("MONGODB_DB_NAME")
            .map_err(|_| MongoError::MissingEnvVar("MONGODB_DB_NAME".to_string()))?;

        let client_options = ClientOptions::parse(mongo_uri)
            .await
            .map_err(MongoError::ParseClientOptionsError)?;
        let client = Client::with_options(client_options).map_err(MongoError::CreateClientError)?;

        client
            .database("admin")
            .run_command(doc! {"ping": 1}, None)
            .await
            .map_err(MongoError::HealthCheckError)?;

        tracing::info!("Connected to MongoDB");

        Ok(Self { client, db_name })
    }

    /// Creates a new MongoDB client with custom URI and database name
    pub async fn with_config(mongo_uri: String, db_name: String) -> Result<Self, MongoError> {
        let client_options = ClientOptions::parse(mongo_uri)
            .await
            .map_err(MongoError::ParseClientOptionsError)?;
        let client = Client::with_options(client_options).map_err(MongoError::CreateClientError)?;

        // Ping the database to verify connection
        client
            .database("admin")
            .run_command(doc! {"ping": 1}, None)
            .await
            .map_err(MongoError::HealthCheckError)?;

        tracing::info!("Connected to MongoDB");

        Ok(Self { client, db_name })
    }

    /// Returns a reference to the MongoDB database
    pub fn database(&self) -> Database {
        self.client.database(&self.db_name)
    }

    /// Returns a typed collection
    pub fn collection<T>(&self, collection_name: &str) -> Collection<T> {
        self.database().collection(collection_name)
    }

    pub async fn insert_one_with_uuid<T: Serialize + DeserializeOwned + Unpin + Send + Sync>(
        &self,
        collection_name: &str,
        uuid: &str,
        document: T,
    ) -> Result<(), MongoError> {
        tracing::debug!(target: "listen-mongo", "Inserting document with UUID: {}", uuid);
        let collection = self.database().collection::<Document>(collection_name);

        // Convert the document to BSON
        let mut doc =
            bson::to_document(&document).map_err(MongoError::ConvertDocumentToBsonError)?;

        // Set the _id field with the provided ID
        doc.insert(
            "_id",
            bson::uuid::Uuid::parse_str(uuid).map_err(MongoError::ParseUuidError)?,
        );

        collection
            .insert_one(doc, None)
            .await
            .map_err(MongoError::InsertError)?;
        Ok(())
    }

    /// Insert a document into the specified collection
    pub async fn insert_one<T: Serialize + DeserializeOwned + Unpin + Send + Sync>(
        &self,
        collection_name: &str,
        document: T,
    ) -> Result<String, MongoError> {
        let collection = self.collection::<T>(collection_name);
        let result = collection
            .insert_one(document, None)
            .await
            .map_err(MongoError::InsertError)?;
        Ok(result
            .inserted_id
            .as_object_id()
            .ok_or_else(|| MongoError::GetInsertedIdError(anyhow!("Failed to get inserted ID")))?
            .to_hex())
    }

    pub async fn find_by_uuid<T: DeserializeOwned + Unpin + Send + Sync>(
        &self,
        collection_name: &str,
        uuid: &str,
    ) -> Result<Option<T>, MongoError> {
        let filter =
            doc! { "_id": bson::uuid::Uuid::parse_str(uuid).map_err(MongoError::ParseUuidError)? };
        tracing::debug!(target: "listen-mongo", "Finding document with UUID: {}", uuid);
        tracing::debug!(target: "listen-mongo", "Filter: {:#?}", filter);
        let collection = self.collection::<T>(collection_name);
        let result = collection
            .find_one(filter, None)
            .await
            .map_err(MongoError::FindError)?;
        Ok(result)
    }

    pub async fn update<T: Serialize + DeserializeOwned + Unpin + Send + Sync>(
        &self,
        collection_name: &str,
        uuid: &str,
        document: T,
    ) -> Result<(), MongoError> {
        let collection = self.collection::<T>(collection_name);
        let filter =
            doc! { "_id": bson::uuid::Uuid::parse_str(uuid).map_err(MongoError::ParseUuidError)? };
        collection
            .update_one(filter, doc! { "$set": bson::to_bson(&document).map_err(MongoError::ConvertDocumentToBsonError)? }, None)
            .await
            .map_err(MongoError::UpdateError)?;
        Ok(())
    }
}

// Example usage
#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, Clone)]
    struct SampleObject {
        name: String,
        email: String,
        age: u32,
    }

    #[tokio::test]
    async fn test_mongo_client() -> Result<()> {
        dotenv::dotenv().ok();
        let repo = MongoClient::from_env().await?;

        // Test with UUID
        let uuid = "123e4567-e89b-12d3-a456-426614174000";
        let user1 = SampleObject {
            name: "John Doe".to_string(),
            email: "john.doe@example.com".to_string(),
            age: 30,
        };
        repo.insert_one_with_uuid("sample_objects", uuid, user1.clone())
            .await?;
        let found_user1 = repo
            .find_by_uuid::<SampleObject>("sample_objects", uuid)
            .await?;
        assert!(found_user1.is_some());
        assert_eq!(found_user1.unwrap().name, "John Doe");

        // cleanup
        repo.database()
            .collection::<SampleObject>("sample_objects")
            .delete_many(doc! {}, None)
            .await?;

        Ok(())
    }
}
