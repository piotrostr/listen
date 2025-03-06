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

impl MongoClient {
    /// Creates a new MongoDB client
    pub async fn from_env() -> Result<Self> {
        let mongo_uri = env::var("MONGODB_URI")
            .map_err(|_| anyhow!("MONGODB_URI not set"))?;
        let db_name = env::var("MONGODB_DB_NAME")
            .map_err(|_| anyhow!("MONGODB_DB_NAME not set"))?;

        let client_options = ClientOptions::parse(mongo_uri).await?;
        let client = Client::with_options(client_options)?;

        client
            .database("admin")
            .run_command(doc! {"ping": 1}, None)
            .await?;

        tracing::info!("Connected to MongoDB");

        Ok(Self { client, db_name })
    }

    /// Creates a new MongoDB client with custom URI and database name
    pub async fn with_config(
        mongo_uri: String,
        db_name: String,
    ) -> Result<Self> {
        let client_options = ClientOptions::parse(mongo_uri).await?;
        let client = Client::with_options(client_options)?;

        // Ping the database to verify connection
        client
            .database("admin")
            .run_command(doc! {"ping": 1}, None)
            .await?;

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

    /// Insert a document into the specified collection
    pub async fn insert_one<
        T: Serialize + DeserializeOwned + Unpin + Send + Sync,
    >(
        &self,
        collection_name: &str,
        document: T,
    ) -> Result<String> {
        let collection = self.collection::<T>(collection_name);
        let result = collection.insert_one(document, None).await?;
        Ok(result
            .inserted_id
            .as_object_id()
            .ok_or_else(|| anyhow!("Failed to get inserted ID"))?
            .to_hex())
    }

    /// Insert a document with a specific key
    pub async fn insert_with_key<
        T: Serialize + DeserializeOwned + Unpin + Send + Sync,
    >(
        &self,
        collection_name: &str,
        key: &str,
        value: T,
    ) -> Result<String> {
        let collection =
            self.database().collection::<Document>(collection_name);

        // Convert value to BSON document
        let serialized_value = bson::to_bson(&value)?;
        let value_doc = match serialized_value {
            bson::Bson::Document(doc) => doc,
            _ => return Err(anyhow!("Failed to convert value to document")),
        };

        // Create document with the specified key
        let mut document = Document::new();
        document.insert(key, value_doc);

        let result = collection.insert_one(document, None).await?;
        Ok(result
            .inserted_id
            .as_object_id()
            .ok_or_else(|| anyhow!("Failed to get inserted ID"))?
            .to_hex())
    }

    /// Find documents in the specified collection
    pub async fn find<T: DeserializeOwned + Unpin + Send + Sync>(
        &self,
        collection_name: &str,
        filter: Document,
        options: Option<FindOptions>,
    ) -> Result<Vec<T>> {
        let collection = self.collection::<Document>(collection_name);
        let cursor = collection.find(filter, options).await?;
        let results = cursor.collect::<Vec<_>>().await;

        let mut documents = Vec::new();
        for result in results {
            match result {
                Ok(doc) => {
                    // Check if the document has fields matching T structure
                    if let Ok(item) = bson::from_document::<T>(doc.clone()) {
                        documents.push(item);
                    } else {
                        // Try to find the object inside nested fields
                        for (_, value) in doc.iter() {
                            if let bson::Bson::Document(nested_doc) = value {
                                if let Ok(item) = bson::from_document::<T>(
                                    nested_doc.clone(),
                                ) {
                                    documents.push(item);
                                    break;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    return Err(anyhow!("Error fetching document: {}", e))
                }
            }
        }

        Ok(documents)
    }

    /// Find a document by ID
    pub async fn find_by_id<T: DeserializeOwned + Unpin + Send + Sync>(
        &self,
        collection_name: &str,
        id: &str,
    ) -> Result<Option<T>> {
        let object_id = bson::oid::ObjectId::parse_str(id)?;
        let filter = doc! { "_id": object_id };

        let collection = self.collection::<T>(collection_name);
        let result = collection.find_one(filter, None).await?;

        Ok(result)
    }
}

// Example usage
#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct SampleObject {
        name: String,
        email: String,
        age: u32,
    }

    #[tokio::test]
    async fn test_mongo_client() -> Result<()> {
        dotenv::dotenv().ok();
        let repo = MongoClient::from_env().await?;
        let user = SampleObject {
            name: "John Doe".to_string(),
            email: "john.doe@example.com".to_string(),
            age: 30,
        };

        let user_id = repo.insert_one("sample_objects", user).await?;
        println!("Inserted user with ID: {}", user_id);

        let found_user = repo
            .find_by_id::<SampleObject>("sample_objects", &user_id)
            .await?;
        println!("Found user: {:?}", found_user);

        let user2 = SampleObject {
            name: "Jane Smith".to_string(),
            email: "jane.smith@example.com".to_string(),
            age: 25,
        };

        let user2_id = repo
            .insert_with_key("sample_objects", "profile", user2)
            .await?;
        println!("Inserted user with key 'profile' and ID: {}", user2_id);

        // Find all users
        let all_users = repo
            .find::<SampleObject>("sample_objects", doc! {}, None)
            .await?;
        println!("Found {} users", all_users.len());

        // cleanup
        repo.database()
            .collection::<SampleObject>("sample_objects")
            .delete_many(doc! {}, None)
            .await?;

        // ensure there are no docs
        let count = repo
            .database()
            .collection::<SampleObject>("sample_objects")
            .count_documents(doc! {}, None)
            .await?;
        assert_eq!(count, 0);

        Ok(())
    }
}
