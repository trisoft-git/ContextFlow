use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProviderResponse {
    pub content: String,
}

#[async_trait]
pub trait IntelligenceProvider: Send + Sync {
    async fn summarize(&self, context: &str) -> Result<ProviderResponse, Box<dyn std::error::Error + Send + Sync>>;
    async fn generate_plan(&self, context: &str) -> Result<ProviderResponse, Box<dyn std::error::Error + Send + Sync>>;
    async fn generate_fix(&self, context: &str, source_context: &str) -> Result<ProviderResponse, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_embeddings(&self, text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>>;
}

pub mod gemini;
pub mod openai;
pub mod anthropic;
pub mod privacy;
