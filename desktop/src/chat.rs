/// Chat client — calls LLM API, runs tool-call loop
/// Returns human-readable progress + final reply
use crate::config::Config;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub struct ChatResult {
    pub content: String,
    pub progress: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct HistoryMessage {
    pub role: String,
    pub content: String,
}

pub async fn exec_with_tools(
    cfg: &Config,
    prompt: &str,
    history: &[HistoryMessage],
) -> Result<ChatResult, Box<dyn std::error::Error + Send + Sync>> {
    let api_key = if !cfg.api_key.is_empty() {
        cfg.api_key.clone()
    } else {
        std::env::var("ANGLES_API_KEY").unwrap_or_default()
    };
    if api_key.is_empty() {
        return Err("API Key 未配置".into());
    }

    let client = Client::new();
    let mut messages: Vec<serde_json::Value> = Vec::new();

    // System prompt
    messages.push(json!({
        "role": "system",
        "content": format!("You are Angles, an AI coding assistant. {}\nLanguage: {}\nYou have tools with angles- prefix.", cfg.agent_persona, cfg.language)
    }));

    for msg in history {
        messages.push(json!({"role": msg.role, "content": msg.content}));
    }
    messages.push(json!({"role": "user", "content": prompt}));

    let mut progress = Vec::new();
    let mut final_content = String::new();

    for _ in 0..20 {
        let url = format!("{}/chat/completions", cfg.base_url.trim_end_matches('/'));
        let body = json!({
            "model": cfg.model,
            "messages": messages,
            "max_tokens": cfg.max_tokens,
            "stream": false,
            "tools": tool_definitions(),
        });

        let res = client.post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send().await?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await?;
            return Err(format!("API {}: {}", status, &text[..text.len().min(500)]).into());
        }

        let data: serde_json::Value = res.json().await?;
        let choice = &data["choices"][0]["message"];
        let content = choice["content"].as_str().unwrap_or("").to_string();
        let tool_calls = choice["tool_calls"].as_array();

        if !content.is_empty() {
            final_content = content.clone();
        }

        messages.push(json!({"role": "assistant", "content": content, "tool_calls": choice["tool_calls"].clone()}));

        if tool_calls.is_none() || tool_calls.unwrap().is_empty() {
            break;
        }

        for tc in tool_calls.unwrap() {
            let name = tc["function"]["name"].as_str().unwrap_or("");
            let raw_args = tc["function"]["arguments"].as_str().unwrap_or("{}");
            let id = tc["id"].as_str().unwrap_or("");

            progress.push(tool_progress(name, raw_args));

            let args: serde_json::Value = serde_json::from_str(raw_args).unwrap_or(json!({}));
            let result = execute_tool(name, &args);

            let truncated = if result.len() > 2000 {
                format!("{}...\n({} chars)", &result[..2000], result.len())
            } else {
                result
            };
            let tc = truncated.clone();
            progress.push(truncated);

            messages.push(json!({"role": "tool", "tool_call_id": id, "content": tc}));
        }
    }

    Ok(ChatResult { content: final_content, progress })
}

fn tool_progress(name: &str, raw_args: &str) -> String {
    let args: serde_json::Value = serde_json::from_str(raw_args).unwrap_or(json!({}));
    let get = |k: &str| -> String { args[k].as_str().unwrap_or("").to_string() };
    match name {
        "angles-createfile" => format!("正在创建 {}", get("path")),
        "angles-writefile"  => format!("正在写入 {}", get("path")),
        "angles-readfile"   => format!("正在读取 {}", get("path")),
        "angles-grep"       => format!("正在搜索 \"{}\"", get("pattern")),
        "angles-ls"         => format!("正在列出 {}", if get("dir").is_empty() { "当前目录".into() } else { get("dir") }),
        "angles-replace"    => format!("正在修改 {}", get("path")),
        "angles-deletefile" => format!("正在删除 {}", get("path")),
        "angles-mkdir"      => format!("正在创建目录 {}", get("path")),
        "angles-run"        => format!("正在执行 {}", get("command")),
        "angles-pwd"        => "正在获取当前路径".into(),
        "angles-fetch"      => format!("正在读取网页 {}", get("url")),
        "angles-websearch"  => format!("正在搜索 \"{}\"", get("query")),
        "angles-gitcommit"  => format!("正在提交 {}", get("msg")),
        _ => format!("正在执行 {}", name),
    }
}

fn execute_tool(name: &str, args: &serde_json::Value) -> String {
    let get = |k: &str| -> String { args[k].as_str().unwrap_or("").to_string() };
    match name {
        "angles-createfile" => {
            let path = get("path");
            let p = std::path::Path::new(&path);
            if p.exists() { return format!("文件已存在: {}", path); }
            if let Some(parent) = p.parent() { let _ = std::fs::create_dir_all(parent); }
            match std::fs::write(p, get("content")) { Ok(_) => format!("已创建: {}", path), Err(e) => format!("失败: {}", e) }
        }
        "angles-writefile" => {
            let p = std::path::Path::new(&get("path"));
            if let Some(parent) = p.parent() { let _ = std::fs::create_dir_all(parent); }
            match std::fs::write(p, get("content")) { Ok(_) => format!("已写入: {}", get("path")), Err(e) => format!("失败: {}", e) }
        }
        "angles-readfile" => {
            match std::fs::read_to_string(get("path")) { Ok(c) => c, Err(e) => format!("失败: {}", e) }
        }
        "angles-ls" => {
            let dir = if get("dir").is_empty() { ".".into() } else { get("dir") };
            match std::fs::read_dir(&dir) {
                Ok(e) => e.filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().into_owned()).collect::<Vec<_>>().join("\n"),
                Err(e) => format!("失败: {}", e),
            }
        }
        "angles-grep" => {
            let dir = if get("directory").is_empty() { ".".into() } else { get("directory") };
            match std::process::Command::new("grep").args(["-rn", &get("pattern"), &dir]).output() {
                Ok(o) => String::from_utf8_lossy(&o.stdout).into_owned(),
                Err(_) => "grep 失败".into(),
            }
        }
        "angles-replace" => {
            let path = get("path");
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    let old_t = get("old"); let new_t = get("new");
                    if content.contains(&old_t) {
                        let replaced = content.replacen(&old_t, &new_t, 1);
                        let _ = std::fs::write(&path, &replaced);
                        format!("已替换: {}", path)
                    } else { format!("未找到: {}", path) }
                }
                Err(e) => format!("失败: {}", e),
            }
        }
        "angles-deletefile" => {
            match std::fs::remove_file(get("path")) { Ok(_) => "已删除".into(), Err(e) => format!("失败: {}", e) }
        }
        "angles-mkdir" => {
            match std::fs::create_dir_all(get("path")) { Ok(_) => "已创建目录".into(), Err(e) => format!("失败: {}", e) }
        }
        "angles-run" => {
            match std::process::Command::new("sh").args(["-c", &get("command")]).output() {
                Ok(o) => {
                    let mut r = String::new();
                    if !o.stdout.is_empty() { r.push_str(&String::from_utf8_lossy(&o.stdout)); }
                    if !o.stderr.is_empty() { r.push_str(&String::from_utf8_lossy(&o.stderr)); }
                    if r.is_empty() { "(无输出)".into() } else { r }
                }
                Err(e) => format!("失败: {}", e),
            }
        }
        "angles-pwd" => std::env::current_dir().map(|d| d.display().to_string()).unwrap_or_else(|e| e.to_string()),
        "angles-fetch" => {
            match std::process::Command::new("curl").args(["-fsSL", "-o", &get("output"), &get("url")]).output() {
                Ok(_) => format!("已下载: {}", get("url")), Err(e) => format!("失败: {}", e),
            }
        }
        "angles-websearch" => format!("搜索: https://www.bing.com/search?q={}", get("query")),
        "angles-gitcommit" => {
            let _ = std::process::Command::new("git").args(["add", "-A"]).output();
            match std::process::Command::new("git").args(["commit", "-m", &get("msg")]).output() {
                Ok(o) if o.status.success() => format!("已提交: {}", get("msg")),
                Ok(o) => format!("失败: {}", String::from_utf8_lossy(&o.stderr)),
                Err(e) => format!("失败: {}", e),
            }
        }
        _ => format!("工具 {} 未实现", name),
    }
}

fn tool_definitions() -> Vec<serde_json::Value> {
    vec![
        json!({"type":"function","function":{"name":"angles-createfile","description":"Create a new file.","parameters":{"type":"object","properties":{"path":{"type":"string"},"content":{"type":"string"}},"required":["path","content"]}}}),
        json!({"type":"function","function":{"name":"angles-writefile","description":"Write to a file.","parameters":{"type":"object","properties":{"path":{"type":"string"},"content":{"type":"string"}},"required":["path","content"]}}}),
        json!({"type":"function","function":{"name":"angles-readfile","description":"Read a file.","parameters":{"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}}}),
        json!({"type":"function","function":{"name":"angles-ls","description":"List directory.","parameters":{"type":"object","properties":{"dir":{"type":"string"}}}}}),
        json!({"type":"function","function":{"name":"angles-grep","description":"Search file contents.","parameters":{"type":"object","properties":{"pattern":{"type":"string"},"directory":{"type":"string"}},"required":["pattern"]}}}),
        json!({"type":"function","function":{"name":"angles-replace","description":"Replace text in file.","parameters":{"type":"object","properties":{"path":{"type":"string"},"old":{"type":"string"},"new":{"type":"string"}},"required":["path","old","new"]}}}),
        json!({"type":"function","function":{"name":"angles-deletefile","description":"Delete a file.","parameters":{"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}}}),
        json!({"type":"function","function":{"name":"angles-mkdir","description":"Create a directory.","parameters":{"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}}}),
        json!({"type":"function","function":{"name":"angles-run","description":"Run a shell command.","parameters":{"type":"object","properties":{"command":{"type":"string"}},"required":["command"]}}}),
        json!({"type":"function","function":{"name":"angles-pwd","description":"Show current directory.","parameters":{"type":"object","properties":{}}}}),
        json!({"type":"function","function":{"name":"angles-fetch","description":"Download a URL.","parameters":{"type":"object","properties":{"url":{"type":"string"},"output":{"type":"string"}},"required":["url","output"]}}}),
        json!({"type":"function","function":{"name":"angles-websearch","description":"Search the web.","parameters":{"type":"object","properties":{"query":{"type":"string"}},"required":["query"]}}}),
        json!({"type":"function","function":{"name":"angles-gitcommit","description":"Git add and commit.","parameters":{"type":"object","properties":{"msg":{"type":"string"}},"required":["msg"]}}}),
    ]
}
