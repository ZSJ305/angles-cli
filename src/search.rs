/// Web search integration for Angles Code CLI.
use crate::config::Config;

pub fn search_url(cfg: &Config, query: &str) -> String {
    let encoded = urlencoding::encode(query);
    match cfg.search_engine.as_str() {
        "bing" => format!("https://www.bing.com/search?q={}", encoded),
        "baidu" => format!("https://www.baidu.com/s?wd={}", encoded),
        "google" => format!("https://www.google.com/search?q={}", encoded),
        "yahoo" => format!("https://search.yahoo.com/search?p={}", encoded),
        "custom" => {
            if cfg.search_engine_url.contains("{q}") {
                cfg.search_engine_url.replace("{q}", &encoded.to_string())
            } else {
                format!("{}{}", cfg.search_engine_url, encoded)
            }
        }
        _ => format!("https://www.bing.com/search?q={}", encoded),
    }
}

/// Build the angles-websearch tool description for the API call.
pub fn websearch_tool() -> serde_json::Value {
    serde_json::json!({
        "type": "function",
        "function": {
            "name": "angles-websearch",
            "description": "Perform a web search using the configured search engine and return result summaries.",
            "parameters": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query string."
                    }
                },
                "required": ["query"]
            }
        }
    })
}

/// Convenience: load config and build search URL (used by api.rs)
pub fn search_url_from_cfg(query: &str) -> String {
    let cfg = crate::config::load_or_default();
    search_url(&cfg, query)
}

// Simple URL encoding (avoid adding another crate for just this)
mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut result = String::new();
        for byte in s.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    result.push(byte as char);
                }
                _ => {
                    result.push_str(&format!("%{:02X}", byte));
                }
            }
        }
        result
    }
}
