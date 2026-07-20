# 🅰 Angles Code CLI

Terminal-based agentic coding assistant. Rust-powered, multi-provider, cross-platform.

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/ZSJ305/angles-cli/main/install.sh | bash
```

## Quick Start

```bash
# 首次配置（交互式向导）
angles gateway

# 开始对话
angles

# 非交互模式
angles exec "写一个 Python HTTP 服务器"

# 查看配置
angles config

# 查看所有命令
angles help

# 诊断
angles doctor
```

## Supported Providers

| Provider | API Host | Protocol |
|---|---|---|
| OpenAI | api.openai.com | OpenAI Chat Completions |
| Claude (Anthropic) | api.anthropic.com | Anthropic Messages API |
| Gemini (Google) | generativelanguage.googleapis.com | Gemini Native API |
| DeepSeek | api.deepseek.com | OpenAI Chat Completions |
| Grok (xAI) | api.x.ai | OpenAI Chat Completions |
| MiniMax | api.minimax.chat | OpenAI Chat Completions |
| OpenRouter | openrouter.ai | OpenAI Chat Completions |
| 通义千问 Qwen | dashscope.aliyuncs.com | OpenAI Chat Completions |
| 智谱 GLM | api.siliconflow.cn | OpenAI Chat Completions |
| Kimi (Moonshot) | api.moonshot.cn | OpenAI Chat Completions |
| Custom | (user-defined) | OpenAI / Anthropic / Gemini |

## Agent Tools

Angles provides 30 built-in `angles-` tool commands:

### File Creation
- `angles-createfile <path> <content>` — Create new file
- `angles-writefile <path> <content>` — Write/overwrite file
- `angles-appendfile <path> <content>` — Append to file
- `angles-insertline <path> <line> <content>` — Insert line

### File Reading
- `angles-readfile <path> [start] [end]` — Read file with optional line range
- `angles-searchfile <pattern> [dir]` — Search files by name (glob)
- `angles-grep <pattern> [dir]` — Search file contents (regex)
- `angles-head <path> [n]` — First n lines
- `angles-tail <path> [n]` — Last n lines

### File Modification
- `angles-replace <path> <old> <new>` — Replace first match
- `angles-replaceall <path> <old> <new>` — Replace all matches
- `angles-deleteline <path> <line>` — Delete line
- `angles-deletefile <path>` — Delete file
- `angles-movedir <src> <dst>` — Move/rename
- `angles-copyfile <src> <dst>` — Copy file
- `angles-mkdir <dir>` — Create directory

### Directory
- `angles-ls [dir]` — List directory
- `angles-tree [dir] [depth]` — Tree view
- `angles-pwd` — Current directory
- `angles-cd <dir>` — Change directory
- `angles-fileinfo <path>` — File metadata

### Terminal
- `angles-run <cmd>` — Execute shell command
- `angles-runbg <cmd>` — Background execution
- `angles-kill <pid>` — Kill process

### Web
- `angles-fetch <url> [output]` — Download URL
- `angles-websearch <query>` — Web search

### Git
- `angles-gitinit [dir]` — Init repo
- `angles-gitcommit <msg>` — Stage all & commit
- `angles-gitlog [n]` — Show commits
- `angles-gitdiff [path]` — Show diff
- `angles-gitbranch <name>` — Create & switch branch

## Setup Wizard (Gateway)

```bash
angles gateway
```

5-step interactive TUI:
1. **Language** — 中文 / English / 日本語
2. **Provider** — 11 providers + custom
3. **Model** — Pre-defined list or manual entry
4. **Preferences** — Max tokens, daily budget, agent persona, approval policy
5. **Search Engine** — Bing / Baidu / Google / Yahoo / Custom URL / Disabled

## Configuration

Stored at `~/.angles/config.json`:

```json
{
  "language": "zh-CN",
  "provider": "glm",
  "base_url": "https://api.siliconflow.cn/v1",
  "wire_api": "chat",
  "model": "zai-org/GLM-5.2",
  "api_key": "",
  "max_tokens": 16384,
  "daily_token_budget": 1000000,
  "agent_persona": "你是一个专业、高效的编码助手。",
  "search_engine": "bing",
  "search_engine_url": "",
  "approval_policy": "untrusted"
}
```

API key can also be set via `ANGLES_API_KEY` env var.

## Build from Source

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone & build
git clone https://github.com/ZSJ305/angles-cli.git
cd angles-cli
cargo build --release

# Install
cp target/release/angles ~/.local/bin/
```

### Cross-compile

```bash
make setup-arm64 && make arm64   # Linux ARM64
make setup-x64 && make x64       # Linux x64
make macos-arm64                  # macOS ARM64 (macOS only)
```

## Architecture

```
angles-cli/
├── src/
│   ├── main.rs          # Entry point & command routing
│   ├── cli.rs           # Clap CLI definitions & help
│   ├── config.rs        # Config load/save/display
│   ├── provider.rs      # 11 provider registry with base URLs
│   ├── gateway.rs       # TUI setup wizard (dialoguer)
│   ├── instructions.rs  # System prompt template rendering (handlebars)
│   ├── api.rs           # API client (OpenAI/Anthropic/Gemini) + streaming + tool loop
│   ├── search.rs        # Web search URL builder
│   └── tools.rs         # 30 angles-* tool implementations + doctor
├── instructions.txt     # System prompt template (13K, {{variable}} injection)
├── AGENTS.md            # Agent tool reference
├── providers.toml       # Provider data source
├── gateway-flow.md      # TUI wizard flow spec
├── Cargo.toml
├── Makefile
├── Cross.toml
├── install.sh
└── .github/workflows/release.yml
```

## License

MIT
