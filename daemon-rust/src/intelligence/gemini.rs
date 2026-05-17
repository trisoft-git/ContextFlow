use async_trait::async_trait;
use serde_json::json;
use crate::intelligence::{IntelligenceProvider, ProviderResponse};

pub struct GeminiProvider {
    api_key: String,
    model_name: String,
    client: reqwest::Client,
}

impl GeminiProvider {
    pub fn new(api_key: String, model_name: Option<String>) -> Self {
        Self {
            api_key,
            model_name: model_name.unwrap_or_else(|| "gemini-2.5-flash".to_string()),
            client: reqwest::Client::new(),
        }
    }

    async fn call_gemini(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model_name, self.api_key
        );

        let body = json!({
            "contents": [{
                "parts": [{
                    "text": prompt
                }]
            }]
        });

        let res = self.client.post(&url)
            .json(&body)
            .send()
            .await?;

        let status = res.status();
        if !status.is_success() {
            let error_text = res.text().await.unwrap_or_else(|_| "Unknown API error".to_string());
            return Err(format!(
                "Gemini API Error (Status {}): {}",
                status, error_text
            ).into());
        }

        let json: serde_json::Value = res.json().await?;
        let text = json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or("Failed to parse Gemini response")?;

        Ok(text.to_string())
    }
}

#[async_trait]
impl IntelligenceProvider for GeminiProvider {
    async fn summarize(&self, context: &str) -> Result<ProviderResponse, Box<dyn std::error::Error + Send + Sync>> {
        let template = crate::prompts::PromptLoader::load("summarize");
        let prompt = crate::prompts::PromptLoader::replace(&template, &[("context", context)]);
        let content = self.call_gemini(&prompt).await?;
        Ok(ProviderResponse { content })
    }

    async fn generate_plan(&self, context: &str) -> Result<ProviderResponse, Box<dyn std::error::Error + Send + Sync>> {
        let template = crate::prompts::PromptLoader::load("plan");
        let prompt = crate::prompts::PromptLoader::replace(&template, &[("context", context)]);
        let content = self.call_gemini(&prompt).await?;
        Ok(ProviderResponse { content })
    }

    async fn generate_fix(&self, context: &str, source_context: &str) -> Result<ProviderResponse, Box<dyn std::error::Error + Send + Sync>> {
        let template = crate::prompts::PromptLoader::load("fix");
        let prompt = crate::prompts::PromptLoader::replace(&template, &[
            ("context", context),
            ("source_files", source_context),
            ("knowledge_context", "") // RAG 연동은 추후 추가
        ]);
        let content = self.call_gemini(&prompt).await?;
        Ok(ProviderResponse { content })
    }

    async fn get_embeddings(&self, text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/text-embedding-004:embedContent?key={}",
            self.api_key
        );

        let body = json!({
            "content": { "parts": [{ "text": text }] }
        });

        let res = self.client.post(&url)
            .json(&body)
            .send()
            .await?;

        let status = res.status();
        if !status.is_success() {
            let error_text = res.text().await.unwrap_or_else(|_| "Unknown API error".to_string());
            return Err(format!(
                "Gemini Embedding API Error (Status {}): {}",
                status, error_text
            ).into());
        }

        let json: serde_json::Value = res.json().await?;
        let values: Vec<f32> = json["embedding"]["values"]
            .as_array()
            .ok_or("Failed to parse embedding response")?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();

        Ok(values)
    }
}
