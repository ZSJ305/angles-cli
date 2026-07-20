/// Gateway TUI setup wizard for Angles Code CLI.
use crate::config::{self, Config};
use crate::provider;

use console::Term;
use dialoguer::{theme::ColorfulTheme, Select, Input, Password, Editor};

pub fn run_wizard() -> Result<(), Box<dyn std::error::Error>> {
    let term = Term::stderr();
    let theme = ColorfulTheme::default();

    println!();
    println!("  ╔═══════════════════════════════════════════╗");
    println!("  ║      🅰  Angles Code CLI Setup            ║");
    println!("  ╚═══════════════════════════════════════════╝");
    println!();

    let mut cfg = Config::default();

    // ── Step 1: Language ──
    println!("  📝 Step 1/5: 选择语言 / Select Language");
    let languages = vec![
        ("中文 (zh-CN)", "zh-CN"),
        ("English (en-US)", "en-US"),
        ("日本語 (ja-JP)", "ja-JP"),
    ];
    let lang_labels: Vec<&str> = languages.iter().map(|(l, _)| *l).collect();
    let lang_idx = Select::with_theme(&theme)
        .items(&lang_labels)
        .default(0)
        .interact_on(&term)?;
    cfg.language = languages[lang_idx].1.to_string();

    let is_zh = cfg.language.starts_with("zh");
    let step2_title = if is_zh { "选择模型提供方" } else { "Select Model Provider" };
    let step2a_title = |name: &str| {
        if is_zh { format!("{} — 输入 API Key", name) } else { format!("{} — Enter API Key", name) }
    };
    let step_custom = if is_zh { "自定义 (Custom)" } else { "Custom" };
    let skip_hint = if is_zh { "(留空跳过，之后通过环境变量配置)" } else { "(Leave empty to skip, configure via env var later)" };

    // ── Step 2: Provider ──
    println!();
    println!("  📝 Step 2/5: {}", step2_title);
    let providers = provider::all_providers();
    let prov_labels: Vec<String> = providers.iter()
        .map(|p| {
            if p.base_url.is_empty() {
                format!("  {}  {}", p.name, step_custom)
            } else {
                let host = p.base_url.replace("https://", "").replace("http://", "").split('/').next().unwrap_or("");
                format!("  {:20} {}", p.name, host)
            }
        })
        .collect();

    let prov_idx = Select::with_theme(&theme)
        .items(&prov_labels)
        .default(8) // GLM as sensible default for Chinese users
        .interact_on(&term)?;

    let selected = &providers[prov_idx];
    cfg.provider = selected.id.clone();
    cfg.base_url = selected.base_url.clone();
    cfg.wire_api = selected.wire_api.clone();
    cfg.model = selected.default_model.clone();

    if selected.id == "custom" {
        // Custom provider: ask for base_url, api_key, model, wire_api
        let base: String = Input::with_theme(&theme)
            .with_prompt("API Base URL")
            .interact_text_on(&term)?;
        cfg.base_url = base;

        let key: String = Password::with_theme(&theme)
            .with_prompt("API Key")
            .allow_empty_password(true)
            .interact_on(&term)?;
        cfg.api_key = key;

        let model: String = Input::with_theme(&theme)
            .with_prompt(if is_zh { "模型 ID" } else { "Model ID" })
            .interact_text_on(&term)?;
        cfg.model = model;

        let wire_options = vec!["OpenAI Chat Completions", "Anthropic Messages", "Gemini Native"];
        let wire_idx = Select::with_theme(&theme)
            .items(&wire_options)
            .default(0)
            .interact_on(&term)?;
        cfg.wire_api = match wire_idx {
            0 => "chat".into(),
            1 => "anthropic".into(),
            2 => "gemini".into(),
            _ => "chat".into(),
        };
    } else {
        // Known provider: just ask for API key
        println!();
        println!("  🔑 {}", step2a_title(&selected.name));
        println!("     API Host: {}", cfg.base_url);
        println!("     {}", skip_hint);
        let key: String = Password::with_theme(&theme)
            .with_prompt("API Key")
            .allow_empty_password(true)
            .interact_on(&term)?;
        cfg.api_key = key;
    }

    // ── Step 3: Model selection ──
    if !providers[prov_idx].models.is_empty() {
        println!();
        println!("  📝 Step 3/5: {}", if is_zh { "选择默认模型" } else { "Select Default Model" });
        let mut model_labels: Vec<String> = providers[prov_idx].models.iter()
            .enumerate()
            .map(|(i, m)| {
                if m == &providers[prov_idx].default_model {
                    format!("  {}  (推荐)", m)
                } else {
                    format!("  {}", m)
                }
            })
            .collect();
        let manual_label = if is_zh { "手动输入模型 ID" } else { "Enter model ID manually" };
        model_labels.push(format!("  {}", manual_label));

        let model_idx = Select::with_theme(&theme)
            .items(&model_labels)
            .default(0)
            .interact_on(&term)?;

        if model_idx < providers[prov_idx].models.len() {
            cfg.model = providers[prov_idx].models[model_idx].clone();
        } else {
            let custom_model: String = Input::with_theme(&theme)
                .with_prompt(if is_zh { "模型 ID" } else { "Model ID" })
                .interact_text_on(&term)?;
            cfg.model = custom_model;
        }
    } else {
        // model already set from step 2 for custom
    }

    // ── Step 4: Preferences ──
    println!();
    println!("  📝 Step 4/5: {}", if is_zh { "偏好设置" } else { "Preferences" });

    let max_tokens: String = Input::with_theme(&theme)
        .with_prompt(if is_zh { "最大输出 Token" } else { "Max Output Tokens" })
        .default("16384".into())
        .interact_text_on(&term)?;
    cfg.max_tokens = max_tokens.parse().unwrap_or(16384);

    let daily_budget: String = Input::with_theme(&theme)
        .with_prompt(if is_zh { "每日 Token 预算" } else { "Daily Token Budget" })
        .default("1000000".into())
        .interact_text_on(&term)?;
    cfg.daily_token_budget = daily_budget.parse().unwrap_or(1_000_000);

    let persona: String = Input::with_theme(&theme)
        .with_prompt(if is_zh { "Agent 人设 (几句话)" } else { "Agent Persona (a few words)" })
        .default(cfg.agent_persona.clone())
        .interact_text_on(&term)?;
    cfg.agent_persona = persona;

    let approval_options = if is_zh {
        vec!["untrusted (安全命令自动执行)", "on-request (agent决定何时问)", "never (全部自动执行)"]
    } else {
        vec!["untrusted (auto-execute safe commands)", "on-request (agent decides)", "never (auto-execute all)"]
    };
    let approval_idx = Select::with_theme(&theme)
        .items(&approval_options)
        .default(0)
        .interact_on(&term)?;
    cfg.approval_policy = match approval_idx {
        0 => "untrusted".into(),
        1 => "on-request".into(),
        2 => "never".into(),
        _ => "untrusted".into(),
    };

    // ── Step 5: Search Engine ──
    println!();
    println!("  📝 Step 5/5: {}", if is_zh { "联网搜索配置" } else { "Web Search Configuration" });

    let search_options = if is_zh {
        vec!["Bing (综合 + AI 摘要)", "Baidu (中文内容)", "Google (覆盖最广)", "Yahoo (备用)", "自定义 URL", "关闭"]
    } else {
        vec!["Bing (general + AI summary)", "Baidu (Chinese content)", "Google (broadest)", "Yahoo (fallback)", "Custom URL", "Disabled"]
    };
    let search_idx = Select::with_theme(&theme)
        .items(&search_options)
        .default(0)
        .interact_on(&term)?;

    cfg.search_engine = match search_idx {
        0 => "bing".into(),
        1 => "baidu".into(),
        2 => "google".into(),
        3 => "yahoo".into(),
        4 => {
            let url: String = Input::with_theme(&theme)
                .with_prompt(if is_zh { "搜索 URL 模板 ({q}=关键词)" } else { "Search URL template ({q}=query)" })
                .interact_text_on(&term)?;
            cfg.search_engine_url = url;
            "custom".into()
        }
        5 => "disabled".into(),
        _ => "bing".into(),
    };

    // ── Save ──
    config::save(&cfg)?;

    println!();
    println!("  ╔═══════════════════════════════════════════╗");
    println!("  ║  ✅ Angles Code CLI 配置完成!              ║");
    println!("  ╚═══════════════════════════════════════════╝");
    println!();
    config::display(&cfg);
    if is_zh {
        println!("  运行 angles 开始对话! 🚀");
    } else {
        println!("  Run `angles` to start chatting! 🚀");
    }
    println!();

    Ok(())
}
