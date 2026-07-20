/// Local HTTP gateway server for Angles Code CLI.
/// Serves a Web Control UI + REST API on 127.0.0.1:8080 (configurable).
///
/// Inspired by OpenClaw's gateway: long-running daemon, local-only bind,
/// control plane + message broker, Web UI for management.
use axum::{
    extract::Json,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;

use crate::config::{self, Config};
use crate::provider;

/// Start the gateway server on the given port.
pub fn start(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        let app = Router::new()
            .route("/", get(control_ui))
            .route("/health", get(health))
            .route("/api/config", get(get_config).put(update_config))
            .route("/api/providers", get(list_providers))
            .route("/api/chat", post(chat))
            .layer(CorsLayer::permissive());

        let addr = format!("127.0.0.1:{}", port);
        println!();
        println!("  ╔═══════════════════════════════════════════╗");
        println!("  ║  🅰  Angles Gateway Server                ║");
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
        "version": "0.1.0",
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
    }).await {
        Ok(Ok(output)) => (StatusCode::OK, Json(serde_json::json!({"reply": output}))),
        Ok(Err(e)) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": format!("Task failed: {}", e)}))),
    }
}

// ─── Web Control UI ───

async fn control_ui() -> Html<&'static str> {
    Html(CONTROL_UI_HTML)
}

const CONTROL_UI_HTML: &str = include_str!("../docs/gateway.html");
