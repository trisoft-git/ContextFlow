use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct KnowledgeManager {
    base_path: PathBuf,
}

impl KnowledgeManager {
    pub fn new() -> Self {
        let mut base_path = std::env::current_dir().expect("Failed to get CWD");
        base_path.push(".contextflow");
        base_path.push("knowledge");

        if !base_path.exists() {
            fs::create_dir_all(&base_path).expect("Failed to create knowledge directory");
        }

        Self { base_path }
    }

    pub fn record_ki(&self, content: &str) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis();
        
        let filename = format!("KI-{}.md", timestamp);
        let file_path = self.base_path.join(filename);
        
        fs::write(&file_path, content)?;
        
        Ok(file_path)
    }

    pub fn list_kis(&self) -> Result<Vec<PathBuf>, std::io::Error> {
        let mut kis = Vec::new();
        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                kis.push(path);
            }
        }
        Ok(kis)
    }
}
