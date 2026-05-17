use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub model: Option<String>,
    pub provider: Option<String>,
    #[serde(rename = "baseUrl")]
    pub base_url: Option<String>,
    pub api_key: Option<String>,
}

pub struct ConfigManager;

impl ConfigManager {
    pub fn load() -> Config {
        // 1. 현재 디렉토리의 .contextflow.json 확인
        let local_path = std::env::current_dir().unwrap().join(".contextflow.json");
        if local_path.exists() {
            if let Ok(content) = fs::read_to_string(local_path) {
                if let Ok(config) = serde_json::from_str::<Config>(&content) {
                    return config;
                }
            }
        }

        // 2. 환경 변수 기반 폴백 (기본값 제공)
        Config {
            model: std::env::var("CF_MODEL").ok(),
            provider: std::env::var("CF_PROVIDER").ok(),
            base_url: std::env::var("CF_BASE_URL").ok(),
            api_key: std::env::var("CF_API_KEY").ok(),
        }
    }

    pub fn save(config: &Config) -> Result<(), std::io::Error> {
        let local_path = std::env::current_dir().unwrap().join(".contextflow.json");
        let content = serde_json::to_string_pretty(config)?;
        fs::write(local_path, content)?;
        Ok(())
    }
}
