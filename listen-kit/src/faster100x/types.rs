use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Faster100xResponse {
    pub result: ResultData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResultData {
    pub data: JsonData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonData {
    pub json: Faster100xData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Faster100xData {
    pub status: String,
    pub message: Option<String>,
    pub data: Option<ResponseData>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseData {
    pub token_address: Option<String>,
    pub response: InnerResponseData,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InnerResponseData {
    pub fund_graph_data: FundGraphData,
    pub data: Vec<Holder>,
    pub top_nodes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FundGraphData {
    pub nodes: Vec<Node>,
    pub links: Vec<Link>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Link {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum PercentageValue {
    String(String),
    Float(f64),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Holder {
    pub address: String,
    pub amount_percentage: PercentageValue,
}

impl Holder {
    pub fn get_percentage(&self) -> f64 {
        match &self.amount_percentage {
            PercentageValue::String(s) => s.parse().unwrap_or(0.0),
            PercentageValue::Float(f) => *f,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HolderRisk {
    pub isolated: IsolatedHolders,
    pub linked: LinkedHolders,
    pub gini_index: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IsolatedHolders {
    pub num_wallets: usize,
    pub total_percentage: f64,
    pub top70_centralization: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkedHolders {
    pub num_clusters: usize,
    pub clusters: Vec<Cluster>,
    pub total_percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cluster {
    pub cluster_wallets: Vec<String>,
    pub num_wallets: usize,
    pub total_percentage: f64,
}
