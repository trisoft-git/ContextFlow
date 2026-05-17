use async_trait::async_trait;
use serde_json::json;
use crate::intelligence::{IntelligenceProvider, ProviderResponse};

pub struct OpenAIProvider {
    api_key: String,
    model_name: String,
    base_url: String,
    client: reqwest::Client,
}

impl OpenAIProvider {
    pub fn new(api_key: String, model_name: Option<String>, base_url: Option<String>) -> Self {
        Self {
            api_key,
            model_name: model_name.unwrap_or_else(|| "gpt-4o".to_string()),
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            client: reqwest::Client::new(),
        }
    }

    async fn call_openai(&self, messages: serde_json::Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/chat/completions", self.base_url);

        let body = json!({
            "model": self.model_name,
            "messages": messages
        });

        let res = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?;

        let status = res.status();
        if !status.is_success() {
            let error_text = res.text().await.unwrap_or_else(|_| "Unknown API error".to_string());
            return Err(format!(
                "OpenAI API Error (Status {}): {}",
                status, error_text
            ).into());
        }

        let json: serde_json::Value = res.json().await?;
        let text = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or("Failed to parse OpenAI response")?;

        Ok(text.to_string())
    }
}

#[async_trait]
impl IntelligenceProvider for OpenAIProvider {
    async fn summarize(&self, context: &str) -> Result<ProviderResponse, Box<dyn std::error::Error + Send + Sync>> {
        let template = crate::prompts::PromptLoader::load("summarize");
        let prompt = crate::prompts::PromptLoader::replace(&template, &[("context", context)]);
        let messages = json!([
            {"role": "system", "content": "You are a developer assistant."},
            {"role": "user", "content": prompt}
        ]);
        let content = self.call_openai(messages).await?;
        Ok(ProviderResponse { content })
    }

    async fn generate_plan(&self, context: &str) -> Result<ProviderResponse, Box<dyn std::error::Error + Send + Sync>> {
        let template = crate::prompts::PromptLoader::load("plan");
        let prompt = crate::prompts::PromptLoader::replace(&template, &[("context", context)]);
        let messages = json!([
            {"role": "system", "content": "You are a technical planner."},
            {"role": "user", "content": prompt}
        ]);
        let content = self.call_openai(messages).await?;
        Ok(ProviderResponse { content })
    }

    async fn generate_fix(&self, context: &str, source_context: &str) -> Result<ProviderResponse, Box<dyn std::error::Error + Send + Sync>> {
        let template = crate::prompts::PromptLoader::load("fix");
        let prompt = crate::prompts::PromptLoader::replace(&template, &[
            ("context", context),
            ("source_files", source_context),
            ("knowledge_context", "")
        ]);
        let messages = json!([{"role": "user", "content": prompt}]);
        let content = self.call_openai(messages).await?;
        Ok(ProviderResponse { content })
    }

    async fn get_embeddings(&self, text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/embeddings", self.base_url);
        let body = json!({
            "model": "text-embedding-3-small",
            "input": text
        });

        let res = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?;

        let status = res.status();
        if !status.is_success() {
            let error_text = res.text().await.unwrap_or_else(|_| "Unknown API error".to_string());
            return Err(format!(
                "OpenAI Embedding API Error (Status {}): {}",
                status, error_text
            ).into());
        }

        let json: serde_json::Value = res.json().await?;
        let values: Vec<f32> = json["data"][0]["embedding"]
            .as_array()
            .ok_or("Failed to parse OpenAI embedding")?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();

        Ok(values)
    }
}
