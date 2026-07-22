/// Provider registry — same as CLI
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub wire_api: String,
    pub default_model: String,
    pub models: Vec<String>,
}

pub fn all_providers() -> Vec<Provider> {
    vec![
        Provider { id: "openai".into(), name: "OpenAI".into(), base_url: "https://api.openai.com/v1".into(), wire_api: "chat".into(), default_model: "gpt-4.1".into(), models: vec!["gpt-4.1".into(), "gpt-5.5".into()] },
        Provider { id: "claude".into(), name: "Claude (Anthropic)".into(), base_url: "https://api.anthropic.com".into(), wire_api: "anthropic".into(), default_model: "claude-sonnet-4-20250514".into(), models: vec!["claude-sonnet-4-20250514".into()] },
        Provider { id: "gemini".into(), name: "Gemini".into(), base_url: "https://generativelanguage.googleapis.com/v1beta".into(), wire_api: "gemini".into(), default_model: "gemini-2.5-pro".into(), models: vec!["gemini-2.5-pro".into()] },
        Provider { id: "deepseek".into(), name: "DeepSeek".into(), base_url: "https://api.deepseek.com/v1".into(), wire_api: "chat".into(), default_model: "deepseek-chat".into(), models: vec!["deepseek-chat".into()] },
        Provider { id: "grok".into(), name: "Grok (xAI)".into(), base_url: "https://api.x.ai/v1".into(), wire_api: "chat".into(), default_model: "grok-4".into(), models: vec!["grok-4".into()] },
        Provider { id: "qwen".into(), name: "Qwen".into(), base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".into(), wire_api: "chat".into(), default_model: "qwen3-235b-a22b".into(), models: vec!["qwen3-235b-a22b".into()] },
        Provider { id: "glm".into(), name: "GLM (Zhipu)".into(), base_url: "https://api.siliconflow.cn/v1".into(), wire_api: "chat".into(), default_model: "zai-org/GLM-5.2".into(), models: vec!["zai-org/GLM-5.2".into()] },
        Provider { id: "kimi".into(), name: "Kimi".into(), base_url: "https://api.moonshot.cn/v1".into(), wire_api: "chat".into(), default_model: "moonshot-v1-auto".into(), models: vec!["moonshot-v1-auto".into()] },
        Provider { id: "openrouter".into(), name: "OpenRouter".into(), base_url: "https://openrouter.ai/api/v1".into(), wire_api: "chat".into(), default_model: "openrouter/auto".into(), models: vec!["openrouter/auto".into()] },
        Provider { id: "custom".into(), name: "Custom".into(), base_url: String::new(), wire_api: "chat".into(), default_model: String::new(), models: vec![] },
    ]
}
