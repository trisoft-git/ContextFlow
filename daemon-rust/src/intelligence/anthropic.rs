use async_trait::async_trait;
use serde_json::json;
use crate::intelligence::{IntelligenceProvider, ProviderResponse};

pub struct AnthropicProvider {
    api_key: String,
    model_name: String,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model_name: Option<String>) -> Self {
        Self {
            api_key,
            model_name: model_name.unwrap_or_else(|| "claude-3-5-sonnet-20240620".to_string()),
            client: reqwest::Client::new(),
        }
    }

    async fn call_anthropic(&self, messages: serde_json::Value, system: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let url = "https://api.anthropic.com/v1/messages";

        let body = json!({
            "model": self.model_name,
            "max_tokens": 4096,
            "system": system,
            "messages": messages
        });

        let res = self.client.post(url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let json: serde_json::Value = res.json().await?;
        let text = json["content"][0]["text"]
            .as_str()
            .ok_or("Failed to parse Anthropic response")?;

        Ok(text.to_string())
    }
}

#[async_trait]
impl IntelligenceProvider for AnthropicProvider {
    async fn summarize(&self, context: &str) -> Result<ProviderResponse, Box<dyn std::error::Error + Send + Sync>> {
        let messages = json!([{"role": "user", "content": context}]);
        let system = "You are a developer assistant. Summarize the activity context.";
        let content = self.call_anthropic(messages, system).await?;
        Ok(ProviderResponse { content })
    }

    async fn generate_plan(&self, context: &str) -> Result<ProviderResponse, Box<dyn std::error::Error + Send + Sync>> {
        let messages = json!([{"role": "user", "content": context}]);
        let system = "Generate a technical execution plan.";
        let content = self.call_anthropic(messages, system).await?;
        Ok(ProviderResponse { content })
    }

    async fn generate_fix(&self, context: &str, source_context: &str) -> Result<ProviderResponse, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = format!("Fix this issue using SEARCH/REPLACE format.\nCONTEXT:\n{}\n\nSOURCE:\n{}", context, source_context);
        let messages = json!([{"role": "user", "content": prompt}]);
        let system = "You are a specialized code fixing agent.";
        let content = self.call_anthropic(messages, system).await?;
        Ok(ProviderResponse { content })
    }

    async fn get_embeddings(&self, _text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>> {
        Err("Anthropic provider does not support embeddings yet. Please use Gemini or OpenAI for RAG features.".into())
    }
}
