use std::fs;

pub struct PromptLoader;

impl PromptLoader {
    pub fn load(name: &str) -> String {
        let mut path = std::env::current_dir().unwrap();
        path.push("prompts");
        path.push(format!("{}.md", name));

        if path.exists() {
            fs::read_to_string(path).unwrap_or_else(|_| "".to_string())
        } else {
            // 기본값 폴백 (최소한의 프롬프트)
            match name {
                "summarize" => "Summarize context: {{context}}".to_string(),
                "fix" => "Fix issue: {{context}} in {{source_files}}".to_string(),
                _ => "".to_string(),
            }
        }
    }

    pub fn replace(template: &str, vars: &[(&str, &str)]) -> String {
        let mut result = template.to_string();
        for (key, value) in vars {
            result = result.replace(&format!("{{{{{}}}}}", key), value);
        }
        result
    }
}
