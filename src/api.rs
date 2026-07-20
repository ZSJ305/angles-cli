/// API client for Angles Code CLI.
/// Handles chat completions (OpenAI-compatible), streaming SSE, and tool execution.
use crate::config::Config;
use crate::instructions;
use crate::search;
use crate::tools;

use futures::StreamExt;
use reqwest::Client;
use serde_json::json;
use std::io::{self, Write};

// ─── Chat Message ───

#[derive(Debug, Clone)]
struct Message {
    role: String,
    content: String,
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone)]
struct ToolCall {
    id: String,
    name: String,
    arguments: String,
}

// ─── Tool definitions for the API ───

fn tool_definitions() -> Vec<serde_json::Value> {
    vec![
        json!({
            "type": "function",
            "function": {
                "name": "angles-createfile",
                "description": "Create a new file with content. Fails if file already exists.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "File path to create" },
                        "content": { "type": "string", "description": "File content" }
                    },
                    "required": ["path", "content"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "angles-writefile",
                "description": "Write content to a file (overwrite). Creates if not exists.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "File path" },
                        "content": { "type": "string", "description": "File content" }
                    },
                    "required": ["path", "content"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "angles-replace",
                "description": "Replace first occurrence of exact text in a file.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "File path" },
                        "old": { "type": "string", "description": "Exact text to find" },
                        "new": { "type": "string", "description": "Replacement text" }
                    },
                    "required": ["path", "old", "new"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "angles-readfile",
                "description": "Read file contents, optionally a line range.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "File path" },
                        "start": { "type": "integer", "description": "Start line (1-based)" },
                        "end": { "type": "integer", "description": "End line" }
                    },
                    "required": ["path"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "angles-grep",
                "description": "Search file contents by pattern (regex supported).",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "pattern": { "type": "string", "description": "Search pattern (regex)" },
                        "directory": { "type": "string", "description": "Directory to search" }
                    },
                    "required": ["pattern"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "angles-searchfile",
                "description": "Search for files by name pattern (glob supported).",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "pattern": { "type": "string", "description": "File name pattern (glob)" },
                        "directory": { "type": "string", "description": "Directory to search" }
                    },
                    "required": ["pattern"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "angles-deletefile",
                "description": "Delete a file permanently.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "File path to delete" }
                    },
                    "required": ["path"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "angles-run",
                "description": "Execute a shell command and return output.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": { "type": "string", "description": "Shell command to run" }
                    },
                    "required": ["command"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "angles-mkdir",
                "description": "Create a directory (with parents).",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Directory path" }
                    },
                    "required": ["path"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "angles-websearch",
                "description": "Perform a web search using the configured search engine.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Search query" }
                    },
                    "required": ["query"]
                }
            }
        }),
    ]
}

// ─── Execute a tool call ───

fn execute_tool(name: &str, args: &serde_json::Value) -> String {
    match name {
        "angles-createfile" => {
            let path = args["path"].as_str().unwrap_or("");
            let content = args["content"].as_str().unwrap_or("");
            tools::angles_createfile(path, content).unwrap_or_else(|e| format!("❌ {}", e))
        }
        "angles-writefile" => {
            let path = args["path"].as_str().unwrap_or("");
            let content = args["content"].as_str().unwrap_or("");
            tools::angles_writefile(path, content).unwrap_or_else(|e| format!("❌ {}", e))
        }
        "angles-replace" => {
            let path = args["path"].as_str().unwrap_or("");
            let old = args["old"].as_str().unwrap_or("");
            let new = args["new"].as_str().unwrap_or("");
            tools::angles_replace(path, old, new).unwrap_or_else(|e| format!("❌ {}", e))
        }
        "angles-readfile" => {
            let path = args["path"].as_str().unwrap_or("");
            let start = args["start"].as_u64().map(|v| v as usize);
            let end = args["end"].as_u64().map(|v| v as usize);
            tools::angles_readfile(path, start, end).unwrap_or_else(|e| format!("❌ {}", e))
        }
        "angles-grep" => {
            let pattern = args["pattern"].as_str().unwrap_or("");
            let directory = args["directory"].as_str().unwrap_or("");
            tools::angles_grep(pattern, directory).unwrap_or_else(|e| format!("❌ {}", e))
        }
        "angles-searchfile" => {
            let pattern = args["pattern"].as_str().unwrap_or("");
            let directory = args["directory"].as_str().unwrap_or("");
            tools::angles_searchfile(pattern, directory).unwrap_or_else(|e| format!("❌ {}", e))
        }
        "angles-deletefile" => {
            let path = args["path"].as_str().unwrap_or("");
            tools::angles_deletefile(path).unwrap_or_else(|e| format!("❌ {}", e))
        }
        "angles-run" => {
            let cmd = args["command"].as_str().unwrap_or("");
            tools::angles_run(cmd).unwrap_or_else(|e| format!("❌ {}", e))
        }
        "angles-mkdir" => {
            let path = args["path"].as_str().unwrap_or("");
            tools::angles_mkdir(path).unwrap_or_else(|e| format!("❌ {}", e))
        }
        "angles-websearch" => {
            let query = args["query"].as_str().unwrap_or("");
            let url = search::search_url_from_cfg(query);
            format!("🔍 搜索: {}", url)
        }
        _ => format!("❌ 未知工具: {}", name),
    }
}

// ─── Start interactive chat (sync wrapper) ───

pub fn start_chat(cfg: Config) -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(start_chat_async(cfg))
}

pub fn exec_once(cfg: Config, prompt: &str) -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(exec_once_async(cfg, prompt))
}

// ─── Async chat implementation ───

async fn start_chat_async(cfg: Config) -> Result<(), Box<dyn std::error::Error>> {
    let system_prompt = instructions::render(&cfg);
    let api_key = if cfg.api_key.is_empty() {
        std::env::var("ANGLES_API_KEY").unwrap_or_default()
    } else {
        cfg.api_key.clone()
    };

    let client = Client::new();
    let mut messages: Vec<Message> = Vec::new();

    println!();
    println!("  🅰  Angles Code CLI v0.1.0");
    println!("  Provider: {} | Model: {}", cfg.provider, cfg.model);
    println!("  输入消息开始对话，/quit 退出，/help 查看命令");
    println!();

    loop {
        print!("  ❯ ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() { continue; }
        if input == "/quit" || input == "/exit" { break; }
        if input == "/help" {
            println!("  /quit, /exit  — 退出");
            println!("  /help         — 显示帮助");
            println!("  /config       — 显示配置");
            println!("  /clear        — 清空对话");
            continue;
        }
        if input == "/config" {
            crate::config::display(&cfg);
            continue;
        }
        if input == "/clear" {
            messages.clear();
            println!("  🗑️  对话已清空");
            continue;
        }

        messages.push(Message {
            role: "user".into(),
            content: input.into(),
            tool_calls: None,
        });

        // API call loop (handles tool calls, max 20 iterations)
        for _ in 0..20 {
            let api_messages = build_api_messages(&system_prompt, &messages);

            let body = json!({
                "model": cfg.model,
                "messages": api_messages,
                "tools": tool_definitions(),
                "tool_choice": "auto",
                "max_tokens": cfg.max_tokens,
                "stream": true,
            });

            let url = format!("{}/chat/completions", cfg.base_url.trim_end_matches('/'));
            let res = client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await?;

            if !res.status().is_success() {
                let status = res.status();
                let text = res.text().await?;
                eprintln!("  ❌ API 错误 {}: {}", status, &text[..text.len().min(500)]);
                break;
            }

            // Parse SSE stream
            let mut stream = res.bytes_stream();
            let mut assistant_content = String::new();
            let mut tool_calls: Vec<ToolCall> = Vec::new();

            print!("  ");
            while let Some(chunk) = stream.next().await {
                let chunk = chunk.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
                let text = String::from_utf8_lossy(&chunk);
                for line in text.lines() {
                    let line = line.trim();
                    if !line.starts_with("data: ") { continue; }
                    let data = &line[6..];
                    if data == "[DONE]" { continue; }

                    let parsed: serde_json::Value = match serde_json::from_str(data) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };

                    let choices = &parsed["choices"];
                    if choices.is_null() || !choices.is_array() { continue; }
                    let delta = &choices[0]["delta"];

                    // Text content
                    if let Some(content) = delta["content"].as_str() {
                        assistant_content.push_str(content);
                        print!("{}", content);
                        io::stdout().flush().ok();
                    }

                    // Tool calls
                    if let Some(tcs) = delta["tool_calls"].as_array() {
                        for tc in tcs {
                            let idx = tc["index"].as_u64().unwrap_or(0) as usize;
                            while tool_calls.len() <= idx {
                                tool_calls.push(ToolCall {
                                    id: String::new(),
                                    name: String::new(),
                                    arguments: String::new(),
                                });
                            }
                            if let Some(id) = tc["id"].as_str() {
                                tool_calls[idx].id = id.to_string();
                            }
                            if let Some(func) = tc.get("function") {
                                if let Some(name) = func["name"].as_str() {
                                    tool_calls[idx].name = name.to_string();
                                }
                                if let Some(args) = func["arguments"].as_str() {
                                    tool_calls[idx].arguments.push_str(args);
                                }
                            }
                        }
                    }
                }
            }
            println!();

            // Add assistant message to history
            messages.push(Message {
                role: "assistant".into(),
                content: assistant_content,
                tool_calls: if tool_calls.is_empty() { None } else { Some(tool_calls.clone()) },
            });

            // If no tool calls, we're done with this turn
            if tool_calls.is_empty() { break; }

            // Execute tool calls and add results
            for tc in &tool_calls {
                let short_args = if tc.arguments.len() > 80 {
                    format!("{}...", &tc.arguments[..80])
                } else {
                    tc.arguments.clone()
                };
                println!("  🔧 {}({})", tc.name, short_args);

                let args: serde_json::Value = serde_json::from_str(&tc.arguments).unwrap_or(json!({}));
                let result = execute_tool(&tc.name, &args);

                // Show first few lines of result
                for line in result.lines().take(5) {
                    println!("  {}", line);
                }
                if result.lines().count() > 5 {
                    println!("  ... ({}行)", result.lines().count());
                }
                println!();

                messages.push(Message {
                    role: "tool".into(),
                    content: result,
                    tool_calls: None,
                });
            }
        }
    }

    println!();
    println!("  👋 再见!");
    Ok(())
}

// ─── Non-interactive exec ───

async fn exec_once_async(cfg: Config, prompt: &str) -> Result<(), Box<dyn std::error::Error>> {
    let system_prompt = instructions::render(&cfg);
    let api_key = if cfg.api_key.is_empty() {
        std::env::var("ANGLES_API_KEY").unwrap_or_default()
    } else {
        cfg.api_key.clone()
    };

    let client = Client::new();
    let api_messages = vec![
        json!({"role": "system", "content": system_prompt}),
        json!({"role": "user", "content": prompt}),
    ];

    let body = json!({
        "model": cfg.model,
        "messages": api_messages,
        "max_tokens": cfg.max_tokens,
        "stream": false,
    });

    let url = format!("{}/chat/completions", cfg.base_url.trim_end_matches('/'));
    let res = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await?;
        eprintln!("❌ API 错误 {}: {}", status, &text[..text.len().min(500)]);
        return Ok(());
    }

    let data: serde_json::Value = res.json().await?;
    if let Some(content) = data["choices"][0]["message"]["content"].as_str() {
        println!("{}", content);
    }

    Ok(())
}

// ─── Helpers ───

fn build_api_messages(system: &str, messages: &[Message]) -> Vec<serde_json::Value> {
    let mut out = vec![json!({"role": "system", "content": system})];
    for msg in messages {
        if msg.role == "tool" {
            out.push(json!({"role": "tool", "content": msg.content}));
        } else if msg.tool_calls.is_some() {
            let tcs: Vec<serde_json::Value> = msg.tool_calls.as_ref().unwrap().iter().map(|tc| {
                json!({
                    "id": tc.id,
                    "type": "function",
                    "function": {
                        "name": tc.name,
                        "arguments": tc.arguments,
                    }
                })
            }).collect();
            out.push(json!({
                "role": "assistant",
                "content": if msg.content.is_empty() { serde_json::Value::Null } else { json!(msg.content) },
                "tool_calls": tcs,
            }));
        } else {
            out.push(json!({"role": msg.role, "content": msg.content}));
        }
    }
    out
}
