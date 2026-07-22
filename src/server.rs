/// Local HTTP gateway server for Angles Code CLI.
/// Serves a Web Control UI + REST API on 127.0.0.1:8080 (configurable).
///
/// Inspired by OpenClaw's gateway: long-running daemon, local-only bind,
/// control plane + message broker, Web UI for management.
use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;

use crate::config::{self, Config};
use crate::provider;

/// Start the gateway server on the given port.
pub fn start(port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        let app = Router::new()
            .route("/", get(control_ui))
            .route("/health", get(health))
            .route("/api/config", get(get_config).put(update_config))
            .route("/api/providers", get(list_providers))
            .route("/api/tools", get(list_tools))
            .route("/api/tools/:name", get(get_tool_detail))
            .route("/api/skills", get(list_skills))
            .route("/api/budget", get(get_budget))
            .route("/api/sessions", get(list_sessions))
            .route("/api/sessions/:id", get(get_session))
            .route("/api/diagnostics", get(diagnostics))
            .route("/api/chat", post(chat))
            .layer(CorsLayer::permissive());

        let addr = format!("127.0.0.1:{}", port);
        println!();
        println!("  ╔═══════════════════════════════════════════╗");
        println!("  ║  α  Angles Gateway Server                ║");
        println!("  ╚═══════════════════════════════════════════╝");
        println!();
        println!("  控制台:  http://127.0.0.1:{}/", port);
        println!("  API:     http://127.0.0.1:{}/api/", port);
        println!("  健康:    http://127.0.0.1:{}/health", port);
        println!();
        println!("  按 Ctrl+C 停止");
        println!();

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;
        Ok(())
    })
}

// ─── API Handlers ───

async fn health() -> impl IntoResponse {
    let cfg = config::load_or_default();
    let key_set = !cfg.api_key.is_empty() || std::env::var("ANGLES_API_KEY").is_ok();
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "provider": cfg.provider,
        "model": cfg.model,
        "api_key_configured": key_set,
        "base_url": cfg.base_url,
    }))
}

async fn get_config() -> impl IntoResponse {
    let cfg = config::load_or_default();
    // Mask API key for security
    let masked_key = if cfg.api_key.is_empty() {
        String::new()
    } else if cfg.api_key.len() > 8 {
        format!("{}...{}", &cfg.api_key[..4], &cfg.api_key[cfg.api_key.len()-4..])
    } else {
        "****".to_string()
    };
    Json(serde_json::json!({
        "language": cfg.language,
        "provider": cfg.provider,
        "base_url": cfg.base_url,
        "wire_api": cfg.wire_api,
        "model": cfg.model,
        "api_key_masked": masked_key,
        "max_tokens": cfg.max_tokens,
        "daily_token_budget": cfg.daily_token_budget,
        "agent_persona": cfg.agent_persona,
        "search_engine": cfg.search_engine,
        "approval_policy": cfg.approval_policy,
    }))
}

#[derive(serde::Deserialize)]
struct UpdateConfigRequest {
    language: Option<String>,
    provider: Option<String>,
    model: Option<String>,
    api_key: Option<String>,
    max_tokens: Option<u32>,
    agent_persona: Option<String>,
    search_engine: Option<String>,
    approval_policy: Option<String>,
}

async fn update_config(Json(req): Json<UpdateConfigRequest>) -> impl IntoResponse {
    let mut cfg = config::load_or_default();

    if let Some(v) = req.language { cfg.language = v; }
    if let Some(v) = req.provider {
        cfg.provider = v.clone();
        // Auto-fill base_url and wire_api from known providers
        if let Some(p) = provider::all_providers().into_iter().find(|p| p.id == v) {
            cfg.base_url = p.base_url.clone();
            cfg.wire_api = p.wire_api.clone();
            if cfg.model.is_empty() { cfg.model = p.default_model.clone(); }
        }
    }
    if let Some(v) = req.model { cfg.model = v; }
    if let Some(v) = req.api_key { cfg.api_key = v; }
    if let Some(v) = req.max_tokens { cfg.max_tokens = v; }
    if let Some(v) = req.agent_persona { cfg.agent_persona = v; }
    if let Some(v) = req.search_engine { cfg.search_engine = v; }
    if let Some(v) = req.approval_policy { cfg.approval_policy = v; }

    match config::save(&cfg) {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"status": "saved"}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

async fn list_providers() -> impl IntoResponse {
    let providers = provider::all_providers();
    let list: Vec<serde_json::Value> = providers.iter().map(|p| {
        serde_json::json!({
            "id": p.id,
            "name": p.name,
            "base_url": p.base_url,
            "wire_api": p.wire_api,
            "default_model": p.default_model,
            "models": p.models,
        })
    }).collect();
    Json(serde_json::json!({"providers": list}))
}

#[derive(serde::Deserialize)]
struct ChatRequest {
    message: String,
}

async fn chat(Json(req): Json<ChatRequest>) -> impl IntoResponse {
    let cfg = config::load_or_default();

    if cfg.api_key.is_empty() && std::env::var("ANGLES_API_KEY").is_err() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "API Key 未配置，请先运行 angles gateway 或通过 API 设置"})),
        );
    }

    // Run the exec in a blocking task to avoid blocking the async runtime
    match tokio::task::spawn_blocking(move || {
        crate::api::exec_once(cfg, &req.message)
            .map_err(|e| e.to_string())
    }).await {
        Ok(Ok(msg)) => (StatusCode::OK, Json(serde_json::json!({"reply": msg}))),
        Ok(Err(e)) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": format!("Task failed: {}", e)}))),
    }
}

// ─── Tools list (static, from source) ───

/// (name, category, description, is_dangerous)
const TOOLS: &[(&str, &str, &str, bool)] = &[
    ("angles-createfile", "文件创建", "创建新文件并写入内容", false),
    ("angles-writefile", "文件创建", "覆盖写入文件", false),
    ("angles-appendfile", "文件创建", "向文件末尾追加", false),
    ("angles-insertline", "文件创建", "指定行号前插入一行", false),
    ("angles-readfile", "读取搜索", "读取文件，可指定起止行", false),
    ("angles-searchfile", "读取搜索", "按文件名 glob 搜索", false),
    ("angles-grep", "读取搜索", "文件内容正则搜索", false),
    ("angles-head", "读取搜索", "显示文件前 n 行", false),
    ("angles-tail", "读取搜索", "显示文件最后 n 行", false),
    ("angles-replace", "修改删除", "精确 diff 替换", false),
    ("angles-replaceall", "修改删除", "替换全部匹配", false),
    ("angles-deleteline", "修改删除", "删除指定行", false),
    ("angles-deletefile", "修改删除", "删除文件（始终确认）", true),
    ("angles-movedir", "修改删除", "移动 / 重命名", false),
    ("angles-copyfile", "修改删除", "复制文件", false),
    ("angles-mkdir", "修改删除", "创建目录", false),
    ("angles-ls", "目录项目", "列出目录内容", false),
    ("angles-tree", "目录项目", "树形结构显示目录", false),
    ("angles-pwd", "目录项目", "显示当前工作目录", false),
    ("angles-cd", "目录项目", "切换工作目录", false),
    ("angles-fileinfo", "目录项目", "文件详细信息", false),
    ("angles-run", "终端执行", "执行命令并返回输出", false),
    ("angles-runbg", "终端执行", "后台执行，返回 PID", false),
    ("angles-kill", "终端执行", "终止指定进程", false),
    ("angles-fetch", "网络搜索", "下载 URL 内容", false),
    ("angles-websearch", "网络搜索", "搜索引擎查询", false),
    ("angles-gitinit", "Git", "初始化 git 仓库", false),
    ("angles-gitcommit", "Git", "暂存并提交", false),
    ("angles-gitlog", "Git", "查看提交记录", false),
    ("angles-gitdiff", "Git", "查看未暂存更改", false),
    ("angles-gitbranch", "Git", "创建并切换分支", false),
];

async fn list_tools() -> impl IntoResponse {
    let list: Vec<serde_json::Value> = TOOLS.iter().map(|(name, cat, desc, danger)| {
        serde_json::json!({
            "name": name,
            "category": cat,
            "description": desc,
            "dangerous": danger,
        })
    }).collect();
    Json(serde_json::json!({"tools": list, "count": list.len()}))
}

async fn get_tool_detail(Path(name): Path<String>) -> impl IntoResponse {
    match TOOLS.iter().find(|(n, _, _, _)| *n == name) {
        Some((n, cat, desc, danger)) => Json(serde_json::json!({
            "name": n, "category": cat, "description": desc, "dangerous": danger
        })),
        None => Json(serde_json::json!({"error": "tool not found"})),
    }
}

// ─── Skills list ───

async fn list_skills() -> impl IntoResponse {
    let skills_dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".angles")
        .join("skills");
    let mut skills: Vec<serde_json::Value> = Vec::new();

    if skills_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&skills_dir) {
            let mut all: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            all.sort_by_key(|e| e.file_name());
            for entry in all {
                if !entry.path().is_dir() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                let skill_md = entry.path().join("SKILL.md");
                let description = if skill_md.exists() {
                    let content = std::fs::read_to_string(&skill_md).unwrap_or_default();
                    extract_skill_description(&content)
                } else {
                    String::new()
                };
                let builtin = name == "skill-creator";
                skills.push(serde_json::json!({
                    "name": name,
                    "description": description,
                    "builtin": builtin,
                }));
            }
        }
    }
    // Ensure builtin exists for API consumers too
    if !skills.iter().any(|s| s["name"] == "skill-creator") {
        crate::skill::ensure_builtin_public();
        skills.insert(0, serde_json::json!({
            "name": "skill-creator",
            "description": "Guide for creating effective skills",
            "builtin": true,
        }));
    }
    Json(serde_json::json!({"skills": skills, "count": skills.len()}))
}

fn extract_skill_description(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut in_frontmatter = false;
    for line in &lines {
        let trimmed = line.trim();
        if trimmed == "---" {
            in_frontmatter = !in_frontmatter;
            if !in_frontmatter { break; }
            continue;
        }
        if in_frontmatter {
            if let Some(rest) = trimmed.strip_prefix("description:") {
                return rest.trim().trim_matches('"').trim_matches('\'').trim().to_string();
            }
        }
    }
    String::new()
}

// ─── Budget / token usage ───

async fn get_budget() -> impl IntoResponse {
    let mut cfg = config::load_or_default();
    config::check_daily_reset(&mut cfg);
    let used = cfg.daily_tokens_used;
    let budget = cfg.daily_token_budget;
    let pct = if budget > 0 { (used as f64 / budget as f64 * 100.0).round() as u64 } else { 0 };
    let remaining = if budget > used { budget - used } else { 0 };
    Json(serde_json::json!({
        "daily_budget": budget,
        "daily_used": used,
        "remaining": remaining,
        "percent_used": pct,
        "reset_date": cfg.daily_reset_date,
    }))
}

// ─── Sessions ───

async fn list_sessions() -> impl IntoResponse {
    let sessions_dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".angles")
        .join("sessions");
    let mut sessions: Vec<serde_json::Value> = Vec::new();
    if sessions_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&sessions_dir) {
            let mut all: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            all.sort_by_key(|e| e.file_name());
            all.reverse(); // newest first
            for entry in all {
                let name = entry.file_name().to_string_lossy().to_string();
                let meta = entry.metadata().ok();
                let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
                let modified = meta.as_ref()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                sessions.push(serde_json::json!({
                    "id": name,
                    "size_bytes": size,
                    "modified_unix": modified,
                }));
            }
        }
    }
    Json(serde_json::json!({"sessions": sessions, "count": sessions.len()}))
}

async fn get_session(Path(id): Path<String>) -> impl IntoResponse {
    let path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".angles")
        .join("sessions")
        .join(&id);
    if !path.exists() {
        return Json(serde_json::json!({"error": "session not found"}));
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => Json(serde_json::json!({"id": id, "content": content})),
        Err(e) => Json(serde_json::json!({"error": e.to_string()})),
    }
}

// ─── Diagnostics ───

async fn diagnostics() -> impl IntoResponse {
    let cfg = config::load_or_default();
    let cfg_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".angles")
        .join("config.json");
    let key_set = !cfg.api_key.is_empty() || std::env::var("ANGLES_API_KEY").is_ok();

    // Test connectivity
    let net_ok = if !cfg.base_url.is_empty() && key_set {
        let test_url = if cfg.base_url.ends_with("/v1") {
            format!("{}/models", cfg.base_url)
        } else {
            format!("{}/v1/models", cfg.base_url.trim_end_matches('/'))
        };
        let key = if !cfg.api_key.is_empty() { cfg.api_key.clone() }
            else { std::env::var("ANGLES_API_KEY").unwrap_or_default() };
        match std::process::Command::new("curl").args([
            "-s", "-o", "/dev/null", "-w", "%{http_code}",
            "--connect-timeout", "5",
            "-H", &format!("Authorization: Bearer {}", key),
            &test_url,
        ]).output() {
            Ok(o) => {
                let code = String::from_utf8_lossy(&o.stdout).trim().to_string();
                code == "200" || code == "401" || code == "403"
            }
            Err(_) => false,
        }
    } else { false };

    // Check git
    let git_ok = std::process::Command::new("git").arg("--version").output().is_ok();
    // Check ripgrep
    let rg_ok = which::which("rg").is_ok();

    Json(serde_json::json!({
        "binary_installed": true,
        "arch": std::env::consts::ARCH,
        "os": std::env::consts::OS,
        "config_exists": cfg_path.exists(),
        "config_path": cfg_path.display().to_string(),
        "api_key_configured": key_set,
        "provider": cfg.provider,
        "model": cfg.model,
        "network_ok": net_ok,
        "git_ok": git_ok,
        "ripgrep_ok": rg_ok,
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

// ─── Web Control UI ───

async fn control_ui() -> Html<&'static str> {
    Html(CONTROL_UI_HTML)
}

const CONTROL_UI_HTML: &str = include_str!("../docs/gateway.html");
