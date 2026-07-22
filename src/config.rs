use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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

fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".angles")
        .join("config.json")
}

pub fn load_or_default() -> Config {
    let path = config_path();
    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(data) => match serde_json::from_str(&data) {
                Ok(cfg) => return cfg,
                Err(e) => eprintln!("⚠ 配置文件解析失败，使用默认值: {}", e),
            },
            Err(e) => eprintln!("⚠ 无法读取配置文件，使用默认值: {}", e),
        }
    }
    Config::default()
}

pub fn save(cfg: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(cfg)?;
    fs::write(&path, data)?;
    Ok(())
}

pub fn display(cfg: &Config) {
    println!();
    println!("  α  Angles Code CLI — 当前配置");
    println!();
    println!("  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  语言:       {}", cfg.language);
    println!("  提供方:     {}", cfg.provider);
    println!("  API Host:   {}", cfg.base_url);
    println!("  协议:       {}", cfg.wire_api);
    println!("  模型:       {}", cfg.model);
    println!("  API Key:    {}",
        if cfg.api_key.is_empty() { "(未设置，使用 ANGLES_API_KEY 环境变量)" } else { "sk-****" }
    );
    println!("  最大Token:  {}", cfg.max_tokens);
    println!("  日预算:     {}", cfg.daily_token_budget);
    println!("  Agent人设:  {}", cfg.agent_persona);
    println!("  搜索引擎:   {}", cfg.search_engine);
    println!("  审批策略:   {}", cfg.approval_policy);
    println!("  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  配置文件:   {}", config_path().display());
    println!();
}

/// Reset daily token counter if the date has changed
pub fn check_daily_reset(cfg: &mut Config) {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    if cfg.daily_reset_date != today {
        cfg.daily_tokens_used = 0;
        cfg.daily_reset_date = today;
        let _ = save(cfg);
    }
}
