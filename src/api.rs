/// Multi-protocol API client for Angles Code CLI.
/// Supports: OpenAI Chat Completions, Anthropic Messages, Gemini Native.
use crate::config::Config;
use crate::instructions;
use crate::search;
use crate::tools;

use futures::StreamExt;
use reqwest::Client;
use serde_json::json;
use std::io::{self, Write};

// ─── Data types ───

#[derive(Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone)]
pub struct ChatResult {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub usage: TokenUsage,
}

#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
}

// ─── Tool definitions ───

pub fn tool_definitions() -> Vec<serde_json::Value> {
    vec![
        json!({"type":"function","function":{"name":"angles-createfile","description":"Create a new file with content. Fails if file already exists.","parameters":{"type":"object","properties":{"path":{"type":"string","description":"File path to create"},"content":{"type":"string","description":"File content"}},"required":["path","content"]}}}),
        json!({"type":"function","function":{"name":"angles-writefile","description":"Write content to a file (overwrite). Creates if not exists.","parameters":{"type":"object","properties":{"path":{"type":"string","description":"File path"},"content":{"type":"string","description":"File content"}},"required":["path","content"]}}}),
        json!({"type":"function","function":{"name":"angles-appendfile","description":"Append content to end of file. Creates if not exists.","parameters":{"type":"object","properties":{"path":{"type":"string","description":"File path"},"content":{"type":"string","description":"Content to append"}},"required":["path","content"]}}}),
        json!({"type":"function","function":{"name":"angles-insertline","description":"Insert a line before the specified line number.","parameters":{"type":"object","properties":{"path":{"type":"string","description":"File path"},"line":{"type":"integer","description":"Line number (1-based)"},"content":{"type":"string","description":"Line content"}},"required":["path","line","content"]}}}),
        json!({"type":"function","function":{"name":"angles-readfile","description":"Read file contents, optionally a line range.","parameters":{"type":"object","properties":{"path":{"type":"string","description":"File path"},"start":{"type":"integer","description":"Start line (1-based)"},"end":{"type":"integer","description":"End line"}},"required":["path"]}}}),
        json!({"type":"function","function":{"name":"angles-head","description":"Show first n lines of a file.","parameters":{"type":"object","properties":{"path":{"type":"string","description":"File path"},"n":{"type":"integer","description":"Number of lines (default 10)"}},"required":["path"]}}}),
        json!({"type":"function","function":{"name":"angles-tail","description":"Show last n lines of a file.","parameters":{"type":"object","properties":{"path":{"type":"string","description":"File path"},"n":{"type":"integer","description":"Number of lines (default 10)"}},"required":["path"]}}}),
        json!({"type":"function","function":{"name":"angles-searchfile","description":"Search for files by name pattern (glob).","parameters":{"type":"object","properties":{"pattern":{"type":"string","description":"File name pattern (glob)"},"directory":{"type":"string","description":"Directory to search"}},"required":["pattern"]}}}),
        json!({"type":"function","function":{"name":"angles-grep","description":"Search file contents by pattern (regex supported).","parameters":{"type":"object","properties":{"pattern":{"type":"string","description":"Search pattern (regex)"},"directory":{"type":"string","description":"Directory to search"}},"required":["pattern"]}}}),
        json!({"type":"function","function":{"name":"angles-replace","description":"Replace first occurrence of exact text in a file.","parameters":{"type":"object","properties":{"path":{"type":"string","description":"File path"},"old":{"type":"string","description":"Exact text to find"},"new":{"type":"string","description":"Replacement text"}},"required":["path","old","new"]}}}),
        json!({"type":"function","function":{"name":"angles-replaceall","description":"Replace all occurrences of exact text in a file.","parameters":{"type":"object","properties":{"path":{"type":"string","description":"File path"},"old":{"type":"string","description":"Exact text to find"},"new":{"type":"string","description":"Replacement text"}},"required":["path","old","new"]}}}),
        json!({"type":"function","function":{"name":"angles-deleteline","description":"Delete a specific line by line number.","parameters":{"type":"object","properties":{"path":{"type":"string","description":"File path"},"line":{"type":"integer","description":"Line number to delete (1-based)"}},"required":["path","line"]}}}),
        json!({"type":"function","function":{"name":"angles-deletefile","description":"Delete a file permanently.","parameters":{"type":"object","properties":{"path":{"type":"string","description":"File path to delete"}},"required":["path"]}}}),
        json!({"type":"function","function":{"name":"angles-copyfile","description":"Copy a file.","parameters":{"type":"object","properties":{"src":{"type":"string","description":"Source path"},"dst":{"type":"string","description":"Destination path"}},"required":["src","dst"]}}}),
        json!({"type":"function","function":{"name":"angles-movedir","description":"Move or rename a file or directory.","parameters":{"type":"object","properties":{"src":{"type":"string","description":"Source path"},"dst":{"type":"string","description":"Destination path"}},"required":["src","dst"]}}}),
        json!({"type":"function","function":{"name":"angles-mkdir","description":"Create a directory (with parents).","parameters":{"type":"object","properties":{"path":{"type":"string","description":"Directory path"}},"required":["path"]}}}),
        json!({"type":"function","function":{"name":"angles-ls","description":"List directory contents.","parameters":{"type":"object","properties":{"dir":{"type":"string","description":"Directory path (default: current)"}}}}}),
        json!({"type":"function","function":{"name":"angles-tree","description":"Show directory tree.","parameters":{"type":"object","properties":{"dir":{"type":"string","description":"Directory path"},"depth":{"type":"integer","description":"Max depth (default 3)"}}}}}),
        json!({"type":"function","function":{"name":"angles-pwd","description":"Show current working directory.","parameters":{"type":"object","properties":{}}}}),
        json!({"type":"function","function":{"name":"angles-cd","description":"Change working directory.","parameters":{"type":"object","properties":{"dir":{"type":"string","description":"Directory path"}},"required":["dir"]}}}),
        json!({"type":"function","function":{"name":"angles-fileinfo","description":"Get file metadata (size, permissions, mtime).","parameters":{"type":"object","properties":{"path":{"type":"string","description":"File path"}},"required":["path"]}}}),
        json!({"type":"function","function":{"name":"angles-run","description":"Execute a shell command and return output.","parameters":{"type":"object","properties":{"command":{"type":"string","description":"Shell command to run"}},"required":["command"]}}}),
        json!({"type":"function","function":{"name":"angles-runbg","description":"Execute a command in background, returns PID.","parameters":{"type":"object","properties":{"command":{"type":"string","description":"Shell command to run"}},"required":["command"]}}}),
        json!({"type":"function","function":{"name":"angles-kill","description":"Kill a process by PID.","parameters":{"type":"object","properties":{"pid":{"type":"integer","description":"Process ID"}},"required":["pid"]}}}),
        json!({"type":"function","function":{"name":"angles-fetch","description":"Download a URL to a file.","parameters":{"type":"object","properties":{"url":{"type":"string","description":"URL to download"},"output":{"type":"string","description":"Output file path"}},"required":["url","output"]}}}),
        json!({"type":"function","function":{"name":"angles-websearch","description":"Perform a web search using the configured search engine.","parameters":{"type":"object","properties":{"query":{"type":"string","description":"Search query"}},"required":["query"]}}}),
        json!({"type":"function","function":{"name":"angles-gitinit","description":"Initialize a git repository.","parameters":{"type":"object","properties":{"dir":{"type":"string","description":"Directory path (default: current)"}}}}}),
        json!({"type":"function","function":{"name":"angles-gitcommit","description":"Stage all changes and commit.","parameters":{"type":"object","properties":{"msg":{"type":"string","description":"Commit message"}},"required":["msg"]}}}),
        json!({"type":"function","function":{"name":"angles-gitlog","description":"Show recent git commits.","parameters":{"type":"object","properties":{"n":{"type":"integer","description":"Number of commits (default 10)"}}}}}),
        json!({"type":"function","function":{"name":"angles-gitdiff","description":"Show git diff (unstaged changes).","parameters":{"type":"object","properties":{"path":{"type":"string","description":"File path (optional)"}}}}}),
        json!({"type":"function","function":{"name":"angles-gitbranch","description":"Create and switch to a new branch.","parameters":{"type":"object","properties":{"name":{"type":"string","description":"Branch name"}},"required":["name"]}}}),
    ]
}

// ─── Tool execution ───

pub fn execute_tool(name: &str, args: &serde_json::Value) -> String {
    match name {
        "angles-createfile" => tools::angles_createfile(args["path"].as_str().unwrap_or(""), args["content"].as_str().unwrap_or("")).unwrap_or_else(|e| format!("❌ {}", e)),
        "angles-writefile" => tools::angles_writefile(args["path"].as_str().unwrap_or(""), args["content"].as_str().unwrap_or("")).unwrap_or_else(|e| format!("❌ {}", e)),
        "angles-appendfile" => tools::angles_appendfile(args["path"].as_str().unwrap_or(""), args["content"].as_str().unwrap_or("")).unwrap_or_else(|e| format!("❌ {}", e)),
        "angles-insertline" => tools::angles_insertline(args["path"].as_str().unwrap_or(""), args["line"].as_u64().unwrap_or(1) as usize, args["content"].as_str().unwrap_or("")).unwrap_or_else(|e| format!("❌ {}", e)),
        "angles-readfile" => tools::angles_readfile(args["path"].as_str().unwrap_or(""), args["start"].as_u64().map(|v| v as usize), args["end"].as_u64().map(|v| v as usize)).unwrap_or_else(|e| format!("❌ {}", e)),
        "angles-head" => tools::angles_head(args["path"].as_str().unwrap_or(""), args["n"].as_u64().unwrap_or(10) as usize).unwrap_or_else(|e| format!("❌ {}", e)),
        "angles-tail" => tools::angles_tail(args["path"].as_str().unwrap_or(""), args["n"].as_u64().unwrap_or(10) as usize).unwrap_or_else(|e| format!("❌ {}", e)),
        "angles-searchfile" => tools::angles_searchfile(args["pattern"].as_str().unwrap_or(""), args["directory"].as_str().unwrap_or("")).unwrap_or_else(|e| format!("❌ {}", e)),
        "angles-grep" => tools::angles_grep(args["pattern"].as_str().unwrap_or(""), args["directory"].as_str().unwrap_or("")).unwrap_or_else(|e| format!("❌ {}", e)),
        "angles-replace" => tools::angles_replace(args["path"].as_str().unwrap_or(""), args["old"].as_str().unwrap_or(""), args["new"].as_str().unwrap_or("")).unwrap_or_else(|e| format!("❌ {}", e)),
        "angles-replaceall" => tools::angles_replaceall(args["path"].as_str().unwrap_or(""), args["old"].as_str().unwrap_or(""), args["new"].as_str().unwrap_or("")).unwrap_or_else(|e| format!("❌ {}", e)),
        "angles-deleteline" => tools::angles_deleteline(args["path"].as_str().unwrap_or(""), args["line"].as_u64().unwrap_or(1) as usize).unwrap_or_else(|e| format!("❌ {}", e)),
        "angles-deletefile" => tools::angles_deletefile(args["path"].as_str().unwrap_or("")).unwrap_or_else(|e| format!("❌ {}", e)),
        "angles-copyfile" => tools::angles_copyfile(args["src"].as_str().unwrap_or(""), args["dst"].as_str().unwrap_or("")).unwrap_or_else(|e| format!("❌ {}", e)),
        "angles-movedir" => tools::angles_movedir(args["src"].as_str().unwrap_or(""), args["dst"].as_str().unwrap_or("")).unwrap_or_else(|e| format!("❌ {}", e)),
        "angles-mkdir" => tools::angles_mkdir(args["path"].as_str().unwrap_or("")).unwrap_or_else(|e| format!("❌ {}", e)),
        "angles-ls" => tools::angles_ls(args["dir"].as_str().unwrap_or("")).unwrap_or_else(|e| format!("❌ {}", e)),
        "angles-tree" => tools::angles_tree(args["dir"].as_str().unwrap_or(""), args["depth"].as_u64().unwrap_or(3) as usize).unwrap_or_else(|e| format!("❌ {}", e)),
        "angles-pwd" => tools::angles_pwd().unwrap_or_else(|e| format!("❌ {}", e)),
        "angles-cd" => tools::angles_cd(args["dir"].as_str().unwrap_or("")).unwrap_or_else(|e| format!("❌ {}", e)),
        "angles-fileinfo" => tools::angles_fileinfo(args["path"].as_str().unwrap_or("")).unwrap_or_else(|e| format!("❌ {}", e)),
        "angles-run" => tools::angles_run(args["command"].as_str().unwrap_or("")).unwrap_or_else(|e| format!("❌ {}", e)),
        "angles-runbg" => tools::angles_runbg(args["command"].as_str().unwrap_or("")).unwrap_or_else(|e| format!("❌ {}", e)),
        "angles-kill" => tools::angles_kill(args["pid"].as_u64().unwrap_or(0) as u32).unwrap_or_else(|e| format!("❌ {}", e)),
        "angles-fetch" => tools::angles_fetch(args["url"].as_str().unwrap_or(""), args["output"].as_str().unwrap_or("")).unwrap_or_else(|e| format!("❌ {}", e)),
        "angles-websearch" => {
            let query = args["query"].as_str().unwrap_or("");
            let url = search::search_url_from_cfg(query);
            format!("🔍 搜索: {}", url)
        }
        "angles-gitinit" => tools::angles_gitinit(args["dir"].as_str().unwrap_or("")).unwrap_or_else(|e| format!("❌ {}", e)),
        "angles-gitcommit" => tools::angles_gitcommit(args["msg"].as_str().unwrap_or("")).unwrap_or_else(|e| format!("❌ {}", e)),
        "angles-gitlog" => tools::angles_gitlog(args["n"].as_u64().unwrap_or(10) as usize).unwrap_or_else(|e| format!("❌ {}", e)),
        "angles-gitdiff" => tools::angles_gitdiff(args["path"].as_str().unwrap_or("")).unwrap_or_else(|e| format!("❌ {}", e)),
        "angles-gitbranch" => tools::angles_gitbranch(args["name"].as_str().unwrap_or("")).unwrap_or_else(|e| format!("❌ {}", e)),
        _ => format!("❌ 未知工具: {}", name),
    }
}

// ════════════════════════════════════════════════════════════
// Protocol: OpenAI Chat Completions
// ════════════════════════════════════════════════════════════

mod openai_chat {
    use super::*;

    pub fn build_request_body(system: &str, messages: &[Message], model: &str, max_tokens: u32) -> serde_json::Value {
        let api_msgs = build_openai_messages(system, messages);
        json!({
            "model": model,
            "messages": api_msgs,
            "tools": tool_definitions(),
            "tool_choice": "auto",
            "max_tokens": max_tokens,
            "stream": true,
        })
    }

    pub fn build_non_stream_body(system: &str, messages: &[Message], model: &str, max_tokens: u32) -> serde_json::Value {
        let api_msgs = build_openai_messages(system, messages);
        json!({
            "model": model,
            "messages": api_msgs,
            "max_tokens": max_tokens,
            "stream": false,
        })
    }

    pub fn build_url(base_url: &str) -> String {
        format!("{}/chat/completions", base_url.trim_end_matches('/'))
    }

    pub fn parse_sse_chunk(data: &str) -> Option<SseEvent> {
        let parsed: serde_json::Value = serde_json::from_str(data).ok()?;
        let choices = parsed.get("choices")?.as_array()?;
        if choices.is_empty() { return None; }
        let delta = &choices[0].get("delta")?;

        let mut content = String::new();
        let mut tool_calls: Vec<ToolCall> = Vec::new();

        if let Some(c) = delta.get("content").and_then(|v| v.as_str()) {
            content = c.to_string();
        }

        if let Some(tcs) = delta.get("tool_calls").and_then(|v| v.as_array()) {
            for tc in tcs {
                let idx = tc.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                while tool_calls.len() <= idx {
                    tool_calls.push(ToolCall { id: String::new(), name: String::new(), arguments: String::new() });
                }
                if let Some(id) = tc.get("id").and_then(|v| v.as_str()) {
                    tool_calls[idx].id = id.to_string();
                }
                if let Some(func) = tc.get("function") {
                    if let Some(name) = func.get("name").and_then(|v| v.as_str()) {
                        tool_calls[idx].name = name.to_string();
                    }
                    if let Some(args) = func.get("arguments").and_then(|v| v.as_str()) {
                        tool_calls[idx].arguments.push_str(args);
                    }
                }
            }
        }

        let usage = TokenUsage {
            prompt_tokens: parsed.get("usage").and_then(|u| u.get("prompt_tokens")).and_then(|v| v.as_u64()).unwrap_or(0),
            completion_tokens: parsed.get("usage").and_then(|u| u.get("completion_tokens")).and_then(|v| v.as_u64()).unwrap_or(0),
        };

        Some(SseEvent { content, tool_calls, usage })
    }

    fn build_openai_messages(system: &str, messages: &[Message]) -> Vec<serde_json::Value> {
        let mut out = vec![json!({"role": "system", "content": system})];
        for msg in messages {
            if msg.role == "tool" {
                out.push(json!({"role": "tool", "content": msg.content}));
            } else if msg.tool_calls.is_some() {
                let tcs: Vec<serde_json::Value> = msg.tool_calls.as_ref().unwrap().iter().map(|tc| {
                    json!({"id": tc.id, "type": "function", "function": {"name": tc.name, "arguments": tc.arguments}})
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
}

// ════════════════════════════════════════════════════════════
// Protocol: Anthropic Messages API
// ════════════════════════════════════════════════════════════

mod anthropic_messages {
    use super::*;

    pub fn build_request_body(system: &str, messages: &[Message], model: &str, max_tokens: u32) -> serde_json::Value {
        let (system_msg, api_msgs) = build_anthropic_messages(system, messages);
        let mut body = json!({
            "model": model,
            "messages": api_msgs,
            "max_tokens": max_tokens,
            "stream": true,
        });
        if !system_msg.is_empty() {
            body["system"] = json!(system_msg);
        }
        // Anthropic tool format uses same structure but different top-level key
        let tools: Vec<serde_json::Value> = tool_definitions().iter().map(|t| {
            json!({
                "name": t["function"]["name"],
                "description": t["function"]["description"],
                "input_schema": t["function"]["parameters"],
            })
        }).collect();
        if !tools.is_empty() {
            body["tools"] = json!(tools);
        }
        body
    }

    pub fn build_url(base_url: &str) -> String {
        format!("{}/v1/messages", base_url.trim_end_matches('/'))
    }

    pub fn parse_sse_chunk(data: &str) -> Option<SseEvent> {
        let parsed: serde_json::Value = serde_json::from_str(data).ok()?;
        let event_type = parsed.get("type").and_then(|v| v.as_str()).unwrap_or("");

        match event_type {
            "content_block_delta" => {
                let delta = parsed.get("delta")?;
                let delta_type = delta.get("type").and_then(|v| v.as_str()).unwrap_or("");
                match delta_type {
                    "text_delta" => {
                        let text = delta.get("text").and_then(|v| v.as_str()).unwrap_or("");
                        Some(SseEvent { content: text.to_string(), tool_calls: Vec::new(), usage: TokenUsage::default() })
                    }
                    "input_json_delta" => {
                        let args = delta.get("partial_json").and_then(|v| v.as_str()).unwrap_or("");
                        Some(SseEvent {
                            content: String::new(),
                            tool_calls: vec![ToolCall { id: String::new(), name: String::new(), arguments: args.to_string() }],
                            usage: TokenUsage::default(),
                        })
                    }
                    _ => None,
                }
            }
            "message_start" => {
                let usage = parsed.pointer("/message/usage")
                    .map(|u| TokenUsage {
                        prompt_tokens: u.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
                        completion_tokens: 0,
                    })
                    .unwrap_or_default();
                Some(SseEvent { content: String::new(), tool_calls: Vec::new(), usage })
            }
            "message_delta" => {
                let usage = parsed.pointer("/usage")
                    .map(|u| TokenUsage {
                        prompt_tokens: 0,
                        completion_tokens: u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
                    })
                    .unwrap_or_default();
                Some(SseEvent { content: String::new(), tool_calls: Vec::new(), usage })
            }
            "tool_use" => {
                // When a content block of type tool_use is started
                let id = parsed.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let name = parsed.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                Some(SseEvent {
                    content: String::new(),
                    tool_calls: vec![ToolCall { id, name, arguments: String::new() }],
                    usage: TokenUsage::default(),
                })
            }
            _ => None,
        }
    }

    /// Convert Angles messages to Anthropic format
    /// Returns (system_prompt, messages_array)
    fn build_anthropic_messages(system: &str, messages: &[Message]) -> (String, Vec<serde_json::Value>) {
        let mut api_msgs = Vec::new();
        for msg in messages {
            if msg.role == "tool" {
                // Tool result → Anthropic tool_result content block
                api_msgs.push(json!({
                    "role": "user",
                    "content": [{"type": "tool_result", "content": msg.content}],
                }));
            } else if msg.tool_calls.is_some() {
                // Assistant with tool calls → content blocks
                let mut blocks: Vec<serde_json::Value> = Vec::new();
                if !msg.content.is_empty() {
                    blocks.push(json!({"type": "text", "text": msg.content}));
                }
                for tc in msg.tool_calls.as_ref().unwrap() {
                    blocks.push(json!({
                        "type": "tool_use",
                        "id": tc.id,
                        "name": tc.name,
                        "input": serde_json::from_str::<serde_json::Value>(&tc.arguments).unwrap_or(json!({})),
                    }));
                }
                api_msgs.push(json!({"role": "assistant", "content": blocks}));
            } else {
                api_msgs.push(json!({"role": msg.role, "content": msg.content}));
            }
        }
        (system.to_string(), api_msgs)
    }
}

// ════════════════════════════════════════════════════════════
// Protocol: Gemini Native API
// ════════════════════════════════════════════════════════════

mod gemini_native {
    use super::*;

    pub fn build_request_body(system: &str, messages: &[Message], model: &str, max_tokens: u32) -> serde_json::Value {
        let contents = build_gemini_contents(messages);
        json!({
            "model": model,
            "contents": contents,
            "systemInstruction": {"parts": [{"text": system}]},
            "generationConfig": {
                "maxOutputTokens": max_tokens,
            },
        })
    }

    pub fn build_url(base_url: &str, model: &str, api_key: &str) -> String {
        let base = base_url.trim_end_matches('/');
        format!("{}/models/{}:streamGenerateContent?alt=sse&key={}", base, model, api_key)
    }

    pub fn parse_sse_chunk(data: &str) -> Option<SseEvent> {
        let parsed: serde_json::Value = serde_json::from_str(data).ok()?;
        let candidates = parsed.get("candidates")?.as_array()?;
        if candidates.is_empty() { return None; }

        let parts = candidates[0].get("content").and_then(|c| c.get("parts")).and_then(|p| p.as_array()).cloned().unwrap_or_default();

        let mut content = String::new();
        let mut tool_calls = Vec::new();

        for part in parts {
            if let Some(text) = part.get("text").and_then(|v| v.as_str()) {
                content.push_str(text);
            }
            if let Some(fc) = part.get("functionCall") {
                tool_calls.push(ToolCall {
                    id: format!("call_{}", tool_calls.len()),
                    name: fc.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    arguments: serde_json::to_string(&fc.get("args").cloned().unwrap_or(json!({}))).unwrap_or_default(),
                });
            }
        }

        Some(SseEvent { content, tool_calls, usage: TokenUsage::default() })
    }

    fn build_gemini_contents(messages: &[Message]) -> Vec<serde_json::Value> {
        let mut out = Vec::new();
        for msg in messages {
            if msg.role == "tool" {
                // Function response
                out.push(json!({
                    "role": "function",
                    "parts": [{"functionResponse": {"name": "tool", "response": {"content": msg.content}}}],
                }));
            } else if msg.tool_calls.is_some() {
                let mut parts: Vec<serde_json::Value> = Vec::new();
                if !msg.content.is_empty() {
                    parts.push(json!({"text": msg.content}));
                }
                for tc in msg.tool_calls.as_ref().unwrap() {
                    let args: serde_json::Value = serde_json::from_str(&tc.arguments).unwrap_or(json!({}));
                    parts.push(json!({"functionCall": {"name": tc.name, "args": args}}));
                }
                out.push(json!({"role": "model", "parts": parts}));
            } else {
                let role = match msg.role.as_str() {
                    "assistant" => "model",
                    "user" => "user",
                    r => r,
                };
                out.push(json!({"role": role, "parts": [{"text": msg.content}]}));
            }
        }
        out
    }
}

// ─── SSE event intermediate ───

struct SseEvent {
    content: String,
    tool_calls: Vec<ToolCall>,
    usage: TokenUsage,
}

// ════════════════════════════════════════════════════════════
// Unified streaming API — dispatches by wire_api
// ════════════════════════════════════════════════════════════

pub fn start_chat(cfg: Config) -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(start_chat_async(cfg))
}

pub fn exec_once(cfg: Config, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(exec_once_async(cfg, prompt))
}

async fn start_chat_async(cfg: Config) -> Result<(), Box<dyn std::error::Error>> {
    let system_prompt = instructions::render(&cfg);
    let api_key = resolve_api_key(&cfg);
    let client = Client::new();
    let mut messages: Vec<Message> = Vec::new();
    let mut daily_used: u64 = 0;

    println!();
    println!("  🅰  Angles Code CLI v0.1.0");
    println!("  Provider: {} | Model: {} | Protocol: {}", cfg.provider, cfg.model, cfg.wire_api);
    println!("  输入消息开始对话，/quit 退出，/help 查看命令");
    println!();

    loop {
        print!("  ❯ ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() { continue; }
        match input {
            "/quit" | "/exit" => break,
            "/help" => {
                println!("  /quit, /exit  — 退出");
                println!("  /help         — 显示帮助");
                println!("  /config       — 显示配置");
                println!("  /clear        — 清空对话");
                println!("  /tokens       — 显示 token 用量");
                continue;
            }
            "/config" => { crate::config::display(&cfg); continue; }
            "/clear" => { messages.clear(); println!("  🗑️  对话已清空"); continue; }
            "/tokens" => { println!("  今日已用: {} / {} tokens", daily_used, cfg.daily_token_budget); continue; }
            _ => {}
        }

        messages.push(Message { role: "user".into(), content: input.into(), tool_calls: None });

        // Tool-call loop (max 20 iterations per user turn)
        for _ in 0..20 {
            let result = stream_turn(&cfg, &system_prompt, &api_key, &client, &messages).await?;

            // Update token usage
            daily_used += result.usage.completion_tokens;

            // Show assistant content
            if !result.content.is_empty() {
                println!("  {}", result.content);
            }

            // Add assistant message to history
            messages.push(Message {
                role: "assistant".into(),
                content: result.content.clone(),
                tool_calls: if result.tool_calls.is_empty() { None } else { Some(result.tool_calls.clone()) },
            });

            // No tool calls → turn done
            if result.tool_calls.is_empty() { break; }

            // Execute tool calls
            for tc in &result.tool_calls {
                let short_args = if tc.arguments.len() > 80 { format!("{}...", &tc.arguments[..80]) } else { tc.arguments.clone() };
                println!("  🔧 {}({})", tc.name, short_args);

                let args: serde_json::Value = serde_json::from_str(&tc.arguments).unwrap_or(json!({}));
                let tool_result = execute_tool(&tc.name, &args);

                // Show first 5 lines
                for (i, line) in tool_result.lines().enumerate() {
                    if i >= 5 { println!("  ... ({}行)", tool_result.lines().count()); break; }
                    println!("  {}", line);
                }
                println!();

                messages.push(Message { role: "tool".into(), content: tool_result, tool_calls: None });
            }

            // Check daily budget
            if daily_used >= cfg.daily_token_budget {
                println!("  ⚠️ 每日 token 预算已用完 ({}/{})", daily_used, cfg.daily_token_budget);
                break;
            }
        }
    }

    println!();
    println!("  👋 再见!");
    Ok(())
}

async fn exec_once_async(cfg: Config, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let system_prompt = instructions::render(&cfg);
    let api_key = resolve_api_key(&cfg);
    let client = Client::new();
    let messages = vec![Message { role: "user".into(), content: prompt.into(), tool_calls: None }];

    let result = stream_turn(&cfg, &system_prompt, &api_key, &client, &messages).await?;
    Ok(result.content)
}

// ─── Single streaming turn — dispatches by wire_api ───

async fn stream_turn(
    cfg: &Config,
    system: &str,
    api_key: &str,
    client: &Client,
    messages: &[Message],
) -> Result<ChatResult, Box<dyn std::error::Error>> {
    match cfg.wire_api.as_str() {
        "chat" => stream_openai_chat(cfg, system, api_key, client, messages).await,
        "anthropic" => stream_anthropic(cfg, system, api_key, client, messages).await,
        "gemini" => stream_gemini(cfg, system, api_key, client, messages).await,
        _ => Err(format!("未知协议: {}", cfg.wire_api).into()),
    }
}

// ─── OpenAI Chat Completions streaming ───

async fn stream_openai_chat(
    cfg: &Config, system: &str, api_key: &str, client: &Client, messages: &[Message],
) -> Result<ChatResult, Box<dyn std::error::Error>> {
    let body = openai_chat::build_request_body(system, messages, &cfg.model, cfg.max_tokens);
    let url = openai_chat::build_url(&cfg.base_url);

    let res = client.post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send().await?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await?;
        return Err(format!("API 错误 {}: {}", status, &text[..text.len().min(500)]).into());
    }

    let mut stream = res.bytes_stream();
    let mut content = String::new();
    let mut tool_calls: Vec<ToolCall> = Vec::new();
    let mut usage = TokenUsage::default();

    print!("  ");
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        let text = String::from_utf8_lossy(&chunk);
        for line in text.lines() {
            let line = line.trim();
            if !line.starts_with("data: ") { continue; }
            let data = &line[6..];
            if data == "[DONE]" { continue; }
            if let Some(event) = openai_chat::parse_sse_chunk(data) {
                if !event.content.is_empty() {
                    content.push_str(&event.content);
                    print!("{}", event.content);
                    io::stdout().flush().ok();
                }
                for tc in event.tool_calls {
                    let idx = tool_calls.len();
                    // Merge into existing tool_call by index tracking
                    // Simple approach: just append (caller will accumulate)
                    tool_calls.push(tc);
                }
                usage.prompt_tokens += event.usage.prompt_tokens;
                usage.completion_tokens += event.usage.completion_tokens;
            }
        }
    }
    println!();

    // Merge tool calls by tracking partial deltas
    tool_calls = merge_tool_calls(tool_calls);

    Ok(ChatResult { content, tool_calls, usage })
}

// ─── Anthropic Messages streaming ───

async fn stream_anthropic(
    cfg: &Config, system: &str, api_key: &str, client: &Client, messages: &[Message],
) -> Result<ChatResult, Box<dyn std::error::Error>> {
    let body = anthropic_messages::build_request_body(system, messages, &cfg.model, cfg.max_tokens);
    let url = anthropic_messages::build_url(&cfg.base_url);

    let res = client.post(&url)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&body)
        .send().await?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await?;
        return Err(format!("Anthropic API 错误 {}: {}", status, &text[..text.len().min(500)]).into());
    }

    let mut stream = res.bytes_stream();
    let mut content = String::new();
    let mut tool_calls: Vec<ToolCall> = Vec::new();
    let mut usage = TokenUsage::default();
    let mut current_tool_id = String::new();
    let mut current_tool_name = String::new();
    let mut current_tool_args = String::new();

    print!("  ");
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        let text = String::from_utf8_lossy(&chunk);
        for line in text.lines() {
            let line = line.trim();
            if !line.starts_with("data: ") { continue; }
            let data = &line[6..];
            if let Some(event) = anthropic_messages::parse_sse_chunk(data) {
                if !event.content.is_empty() {
                    content.push_str(&event.content);
                    print!("{}", event.content);
                    io::stdout().flush().ok();
                }
                for tc in event.tool_calls {
                    if !tc.id.is_empty() { current_tool_id = tc.id; }
                    if !tc.name.is_empty() { current_tool_name = tc.name; }
                    current_tool_args.push_str(&tc.arguments);
                }
                usage.prompt_tokens += event.usage.prompt_tokens;
                usage.completion_tokens += event.usage.completion_tokens;
            }
        }
    }
    println!();

    // Finalize any pending tool call
    if !current_tool_name.is_empty() {
        tool_calls.push(ToolCall {
            id: current_tool_id,
            name: current_tool_name,
            arguments: current_tool_args,
        });
    }

    Ok(ChatResult { content, tool_calls, usage })
}

// ─── Gemini Native streaming ───

async fn stream_gemini(
    cfg: &Config, system: &str, api_key: &str, client: &Client, messages: &[Message],
) -> Result<ChatResult, Box<dyn std::error::Error>> {
    let body = gemini_native::build_request_body(system, messages, &cfg.model, cfg.max_tokens);
    let url = gemini_native::build_url(&cfg.base_url, &cfg.model, api_key);

    let res = client.post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send().await?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await?;
        return Err(format!("Gemini API 错误 {}: {}", status, &text[..text.len().min(500)]).into());
    }

    let mut stream = res.bytes_stream();
    let mut content = String::new();
    let mut tool_calls: Vec<ToolCall> = Vec::new();
    let mut usage = TokenUsage::default();

    print!("  ");
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        let text = String::from_utf8_lossy(&chunk);
        for line in text.lines() {
            let line = line.trim();
            if !line.starts_with("data: ") { continue; }
            let data = &line[6..];
            if let Some(event) = gemini_native::parse_sse_chunk(data) {
                if !event.content.is_empty() {
                    content.push_str(&event.content);
                    print!("{}", event.content);
                    io::stdout().flush().ok();
                }
                tool_calls.extend(event.tool_calls);
                usage.prompt_tokens += event.usage.prompt_tokens;
                usage.completion_tokens += event.usage.completion_tokens;
            }
        }
    }
    println!();

    Ok(ChatResult { content, tool_calls, usage })
}

// ─── Helpers ───

fn resolve_api_key(cfg: &Config) -> String {
    if !cfg.api_key.is_empty() {
        cfg.api_key.clone()
    } else {
        std::env::var("ANGLES_API_KEY").unwrap_or_default()
    }
}

/// Merge partial tool call deltas into complete tool calls.
/// OpenAI sends tool_calls across multiple SSE chunks with incrementing indices.
fn merge_tool_calls(partial: Vec<ToolCall>) -> Vec<ToolCall> {
    if partial.is_empty() { return partial; }
    // Simple: if we have multiple with same empty id, merge args
    let mut result: Vec<ToolCall> = Vec::new();
    for tc in partial {
        if let Some(last) = result.last_mut() {
            // If the new tc has no id and last has no name yet, it's a continuation
            if tc.id.is_empty() && !tc.name.is_empty() && last.name.is_empty() {
                last.name = tc.name.clone();
            }
            if tc.id.is_empty() && tc.name.is_empty() && !tc.arguments.is_empty() {
                last.arguments.push_str(&tc.arguments);
            }
            if !tc.id.is_empty() {
                result.push(tc);
            }
        } else {
            result.push(tc);
        }
    }
    result
}
