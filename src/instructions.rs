use crate::config::Config;
use handlebars::Handlebars;

/// Render the instructions template with config values injected.
pub fn render(cfg: &Config) -> String {
    let template = include_str!("../instructions.txt");
    let mut hb = Handlebars::new();
    hb.register_escape_fn(handlebars::no_escape);
    hb.register_template_string("instructions", template).unwrap();

    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;

    let data = serde_json::json!({
        "arch": arch,
        "os": os,
        "language": cfg.language,
        "provider": cfg.provider,
        "base_url": cfg.base_url,
        "model": cfg.model,
        "max_tokens": cfg.max_tokens,
        "daily_token_budget": cfg.daily_token_budget,
        "agent_persona": cfg.agent_persona,
        "search_engine": cfg.search_engine,
        "approval_policy": cfg.approval_policy,
    });

    hb.render("instructions", &data).unwrap_or_else(|e| {
        eprintln!("⚠ Instructions 模板渲染失败: {}", e);
        template.to_string()
    })
}
