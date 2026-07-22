/// Config — shared format with angles-cli
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub language: String,
    pub provider: String,
    pub base_url: String,
    pub wire_api: String,
    pub model: String,
    pub api_key: String,
    pub max_tokens: u32,
    pub daily_token_budget: u64,
    pub agent_persona: String,
    pub search_engine: String,
    #[serde(default)]
    pub search_engine_url: String,
    pub approval_policy: String,
    #[serde(default)]
    pub daily_tokens_used: u64,
    #[serde(default)]
    pub daily_reset_date: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            language: "zh-CN".into(),
            provider: "glm".into(),
            base_url: "https://api.siliconflow.cn/v1".into(),
            wire_api: "chat".into(),
            model: "zai-org/GLM-5.2".into(),
            api_key: String::new(),
            max_tokens: 16384,
            daily_token_budget: 1_000_000,
            agent_persona: "你是一个专业、高效的编码助手。".into(),
            search_engine: "bing".into(),
            search_engine_url: String::new(),
            approval_policy: "untrusted".into(),
            daily_tokens_used: 0,
            daily_reset_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
        }
    }
}

pub fn load_or_default(path: &Path) -> Config {
    if path.exists() {
        if let Ok(data) = fs::read_to_string(path) {
            if let Ok(cfg) = serde_json::from_str(&data) {
                return cfg;
            }
        }
    }
    Config::default()
}

pub fn save(cfg: &Config, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(cfg)?)?;
    Ok(())
}
