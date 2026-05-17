use regex::Regex;
use std::sync::OnceLock;

pub struct PrivacyFilter;

impl PrivacyFilter {
    pub fn mask_sensitive_data(text: &str) -> String {
        static REGEXES: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();
        let regex_list = REGEXES.get_or_init(|| {
            vec![
                // 1. API Keys, passwords, tokens: e.g. api_key = "abc...", token: 'xyz...', secret_key : '...'
                (Regex::new(r#"(?i)(api[_-]?key|secret|password|token)\s*[:=]\s*['"]?[a-zA-Z0-9_\-\.~\+]{16,}['"]?"#).unwrap(), "$1 = \"[REDACTED_SECRET]\""),
                // 2. Bearer tokens
                (Regex::new(r#"(?i)(bearer\s+[a-zA-Z0-9_\-\.~\+]{16,})"#).unwrap(), "Bearer [REDACTED_BEARER]"),
                // 3. Private SSH keys
                (Regex::new(r#"-----BEGIN [A-Z ]+ PRIVATE KEY-----[a-zA-Z0-9\+/\s\n\r=]+-----END [A-Z ]+ PRIVATE KEY-----"#).unwrap(), "[REDACTED_PRIVATE_KEY]"),
            ]
        });

        let mut result = text.to_string();
        for (re, replacement) in regex_list {
            result = re.replace_all(&result, *replacement).to_string();
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privacy_filter_masking() {
        let input = "export GEMINI_API_KEY=\"AIzaSyD-1234567890abcdefghijklmnopqrstuvwxyz\"\nlet token = 'xyz_1234567890abcdefghijklmnopqrstuvwxyz';\nAuthorization: Bearer my_super_secret_token_1234567890";
        let masked = PrivacyFilter::mask_sensitive_data(input);
        assert!(masked.contains("GEMINI_API_KEY = \"[REDACTED_SECRET]\""));
        assert!(masked.contains("token = \"[REDACTED_SECRET]\""));
        assert!(masked.contains("Authorization: Bearer [REDACTED_BEARER]"));
    }
}
