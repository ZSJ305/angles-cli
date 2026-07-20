use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub wire_api: String,
    pub default_model: String,
    pub models: Vec<String>,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Deserialize)]
struct ProvidersFile {
    providers: Vec<Provider>,
}

pub fn all_providers() -> Vec<Provider> {
    vec![
        Provider {
            id: "openai".into(),
            name: "OpenAI".into(),
            base_url: "https://api.openai.com/v1".into(),
            wire_api: "chat".into(),
            default_model: "gpt-4.1".into(),
            models: vec!["gpt-4.1".into(), "gpt-4.1-mini".into(), "gpt-4.1-nano".into(), "o3".into(), "o3-mini".into(), "o4-mini".into(), "gpt-5.5".into()],
            note: String::new(),
        },
        Provider {
            id: "claude".into(),
            name: "Claude (Anthropic)".into(),
            base_url: "https://api.anthropic.com".into(),
            wire_api: "anthropic".into(),
            default_model: "claude-sonnet-4-20250514".into(),
            models: vec!["claude-sonnet-4-20250514".into(), "claude-opus-4-20250514".into(), "claude-haiku-3-5-20241022".into()],
            note: "Anthropic 使用 Messages API 协议".into(),
        },
        Provider {
            id: "gemini".into(),
            name: "Gemini (Google)".into(),
            base_url: "https://generativelanguage.googleapis.com/v1beta".into(),
            wire_api: "gemini".into(),
            default_model: "gemini-2.5-pro".into(),
            models: vec!["gemini-2.5-pro".into(), "gemini-2.5-flash".into(), "gemini-2.0-flash".into()],
            note: "Gemini 使用原生 API 协议".into(),
        },
        Provider {
            id: "deepseek".into(),
            name: "DeepSeek".into(),
            base_url: "https://api.deepseek.com/v1".into(),
            wire_api: "chat".into(),
            default_model: "deepseek-chat".into(),
            models: vec!["deepseek-chat".into(), "deepseek-reasoner".into()],
            note: String::new(),
        },
        Provider {
            id: "grok".into(),
            name: "Grok (xAI)".into(),
            base_url: "https://api.x.ai/v1".into(),
            wire_api: "chat".into(),
            default_model: "grok-4".into(),
            models: vec!["grok-4".into(), "grok-3".into(), "grok-3-mini".into()],
            note: String::new(),
        },
        Provider {
            id: "minimax".into(),
            name: "MiniMax".into(),
            base_url: "https://api.minimax.chat/v1".into(),
            wire_api: "chat".into(),
            default_model: "MiniMax-M1".into(),
            models: vec!["MiniMax-M1".into(), "abab6.5s-chat".into()],
            note: String::new(),
        },
        Provider {
            id: "openrouter".into(),
            name: "OpenRouter".into(),
            base_url: "https://openrouter.ai/api/v1".into(),
            wire_api: "chat".into(),
            default_model: "openrouter/auto".into(),
            models: vec!["openrouter/auto".into(), "anthropic/claude-sonnet-4".into(), "openai/gpt-4.1".into(), "google/gemini-2.5-pro".into()],
            note: "OpenRouter 可通过 model ID 访问多个提供方".into(),
        },
        Provider {
            id: "qwen".into(),
            name: "通义千问 (Qwen)".into(),
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".into(),
            wire_api: "chat".into(),
            default_model: "qwen3-235b-a22b".into(),
            models: vec!["qwen3-235b-a22b".into(), "qwen3-32b".into(), "qwen3-30b-a3b".into(), "qwen-max".into(), "qwen-plus".into()],
            note: String::new(),
        },
        Provider {
            id: "glm".into(),
            name: "智谱 GLM".into(),
            base_url: "https://api.siliconflow.cn/v1".into(),
            wire_api: "chat".into(),
            default_model: "zai-org/GLM-5.2".into(),
            models: vec!["zai-org/GLM-5.2".into(), "zai-org/GLM-4.6".into(), "THUDM/GLM-4-32B".into()],
            note: "SiliconFlow 托管".into(),
        },
        Provider {
            id: "kimi".into(),
            name: "Kimi (Moonshot)".into(),
            base_url: "https://api.moonshot.cn/v1".into(),
            wire_api: "chat".into(),
            default_model: "moonshot-v1-auto".into(),
            models: vec!["moonshot-v1-auto".into(), "moonshot-v1-8k".into(), "moonshot-v1-32k".into(), "moonshot-v1-128k".into(), "kimi-latest".into()],
            note: String::new(),
        },
        Provider {
            id: "custom".into(),
            name: "自定义 (Custom)".into(),
            base_url: String::new(),
            wire_api: "chat".into(),
            default_model: String::new(),
            models: vec![],
            note: "需手动填写 base_url 和 model".into(),
        },
    ]
}

pub fn find_by_id(id: &str) -> Option<Provider> {
    all_providers().into_iter().find(|p| p.id == id)
}
