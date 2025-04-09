use crate::completion::generate_completion;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryNote {
    pub id: Uuid,
    pub content: String,

    // Semantic metadata
    pub keywords: Vec<String>,
    pub links: Vec<Uuid>, // Representing links as Vec<String> of IDs for simplicity
    pub context: String,
    pub category: String,
    pub tags: Vec<String>,

    // Temporal information
    pub timestamp: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,

    // Usage and evolution data
    pub retrieval_count: u32,
    pub evolution_history: Vec<String>, // Representing history as Vec<String> for simplicity
}

impl MemoryNote {
    pub fn new(content: String) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::new_v4(),
            content: content.clone(),
            keywords: Vec::new(),
            links: Vec::new(),
            context: "General".to_string(),        // Default context
            category: "Uncategorized".to_string(), // Default category
            tags: Vec::new(),
            timestamp: now,
            last_accessed: now,
            retrieval_count: 0,
            evolution_history: Vec::new(),
        }
    }

    pub async fn with_llm_analysis(content: String) -> Result<Self> {
        let mut note = Self::new(content.clone());

        // Generate the analysis prompt
        let prompt = format!(
            r#"Generate a structured analysis of the following content by:
            1. Identifying the most salient keywords (focus on nouns, verbs, and key concepts)
            2. Extracting core themes and contextual elements
            3. Creating relevant categorical tags

            Format the response as a JSON object:
            {{
                "keywords": [
                    // several specific, distinct keywords that capture key concepts and terminology
                    // Order from most to least important
                    // Don't include keywords that are the name of the speaker or time
                    // At least three keywords, but don't be too redundant.
                ],
                "context": 
                    // one sentence summarizing:
                    // - Main topic/domain
                    // - Key arguments/points
                    // - Intended audience/purpose
                ,
                "tags": [
                    // several broad categories/themes for classification
                    // Include domain, format, and type tags
                    // At least three tags, but don't be too redundant.
                ]
            }}

            Content for analysis:
            {}"#,
            content
        );

        // Call the completion API
        let response = generate_completion(&prompt).await?;

        // Parse the response
        let analysis: Value = serde_json::from_str(&response)?;

        // Update note with the analysis results
        if let Some(keywords) = analysis["keywords"].as_array() {
            note.keywords = keywords
                .iter()
                .filter_map(|k| k.as_str().map(String::from))
                .collect();
        }

        if let Some(context) = analysis["context"].as_str() {
            note.context = context.to_string();
        }

        if let Some(tags) = analysis["tags"].as_array() {
            note.tags = tags
                .iter()
                .filter_map(|t| t.as_str().map(String::from))
                .collect();
        }

        Ok(note)
    }

    pub fn to_metadata(&self) -> HashMap<String, Value> {
        let mut metadata = HashMap::new();

        metadata.insert("id".to_string(), json!(self.id.to_string()));
        metadata.insert("keywords".to_string(), json!(self.keywords));
        metadata.insert("context".to_string(), json!(self.context));
        metadata.insert("category".to_string(), json!(self.category));
        metadata.insert("tags".to_string(), json!(self.tags));
        metadata.insert("timestamp".to_string(), json!(self.timestamp.to_rfc3339()));
        metadata.insert(
            "last_accessed".to_string(),
            json!(self.last_accessed.to_rfc3339()),
        );
        metadata.insert("retrieval_count".to_string(), json!(self.retrieval_count));

        metadata
    }

    pub fn from_metadata(
        content: String,
        metadata: HashMap<String, Value>,
    ) -> Result<Self, String> {
        let id = match metadata.get("id") {
            Some(Value::String(id_str)) => match Uuid::parse_str(id_str) {
                Ok(uuid) => uuid,
                Err(_) => return Err("Invalid UUID format".to_string()),
            },
            _ => Uuid::new_v4(), // Generate new if not found or invalid
        };

        let keywords = metadata
            .get("keywords")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let context = metadata
            .get("context")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| "General".to_string());

        let category = metadata
            .get("category")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| "Uncategorized".to_string());

        let tags = metadata
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        // Parse timestamps or use current time
        let now = Utc::now();
        let timestamp = metadata
            .get("timestamp")
            .and_then(|v| v.as_str())
            .and_then(|ts| DateTime::parse_from_rfc3339(ts).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(now);

        let last_accessed = metadata
            .get("last_accessed")
            .and_then(|v| v.as_str())
            .and_then(|ts| DateTime::parse_from_rfc3339(ts).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(now);

        let retrieval_count = metadata
            .get("retrieval_count")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(0);

        Ok(Self {
            id,
            content,
            keywords,
            links: Vec::new(), // Links require special handling
            context,
            category,
            tags,
            timestamp,
            last_accessed,
            retrieval_count,
            evolution_history: Vec::new(),
        })
    }

    pub fn increment_retrieval_count(&mut self) {
        self.retrieval_count += 1;
        self.last_accessed = Utc::now();
    }

    pub fn add_to_evolution_history(&mut self, event: String) {
        self.evolution_history.push(event);
    }
}
