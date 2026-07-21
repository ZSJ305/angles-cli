<div align="center">

# 🅰 Angles Code CLI

### 1.6 MB 的 code agent — Rust 单二进制，不装运行时

**30 个内置工具 · 11 家模型供应商 · 5 平台预编译 · MIT 开源**

<p>
  <a href="https://zsj305.github.io/angles-cli/">🏠 官网</a> ·
  <a href="https://zsj305.github.io/angles-cli/docs.html">📖 技术文档</a> ·
  <a href="https://zsj305.github.io/angles-cli/tools.html">🔧 工具速查</a> ·
  <a href="https://zsj305.github.io/angles-cli/faq.html">❓ FAQ</a> ·
  <a href="https://www.npmjs.com/package/@angleschina/angles">📦 npm</a> ·
  <a href="https://github.com/ZSJ305/angles-cli/releases">⬇️ Releases</a>
</p>

```bash
npm i -g @angleschina/angles && angles install
```

| | |
|---|---|
| 📦 二进制大小 | **1.6 MB**（musl 静态 / MSVC） |
| 🧩 内置工具 | **30+** （文件 / 目录 / 终端 / Git / 网络） |
| 🤖 模型供应商 | **11 家**（OpenAI / Claude / Gemini / DeepSeek / Grok / Qwen / GLM / Kimi …） |
| 🖥️ 预编译平台 | **5 个**（Linux ARM64/x64 · macOS ARM64/x64 · Windows x64） |
| ⚡ 运行时依赖 | **0**（纯 Rust，零 Node / 零 Python / 零 libc 动态链接 musl） |
| 🔓 许可证 | **MIT** |

</div>

---

一个终端里的 agentic 编码助手。Rust 编译成单个静态二进制——不装 Node、不装运行时、不装编译器，一行命令装好就开干。agent 通过 30 个 `angles-*` 工具直接操作文件、目录、终端、Git、网络，配 11 家模型供应商任意切换。配置存在本地 `~/.angles/`，API Key 不经任何中转。

## Install

四种方式，装到的二进制完全相同：

<table>
<tr>
<td>

**npm（推荐）**

```bash
npm i -g @angleschina/angles && angles install
```

国内默认走 npmmirror 镜像，最快。

</td>
<td>

**curl 一行（Linux / macOS / WSL2）**

```bash
curl -fsSL https://zsj305.github.io/angles-cli/install.sh | bash
```

</td>
</tr>
<tr>
<td>

**PowerShell 一行（Windows）**

```powershell
irm https://zsj305.github.io/angles-cli/install.ps1 | iex
```

</td>
<td>

**wget 一行**

```bash
wget -qO- https://zsj305.github.io/angles-cli/install.sh | bash
```

</td>
</tr>
</table>

预编译可用时安装器会跳过 Rust / 编译工具链，几秒下一颗 ~1.6 MB 二进制就装好。iSH、树莓派、Alpine、无 root 环境都能直接装。

## Quick Start

```bash
# 首次配置（5 步 TUI 向导）
angles gateway

# 进入对话（默认模式）
angles

# 非交互执行
angles exec "写一个 Python HTTP 服务器"

# 启动本地 HTTP 网关 (v0.2+) — 浏览器开 http://127.0.0.1:8080
angles serve

# 查看配置 / 诊断 / 帮助
angles config
angles doctor
angles help
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

Angles provides 30+ built-in `angles-` tool commands:

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

## Local HTTP Gateway (v0.2+)

```bash
angles serve                 # 启动 8080
angles serve --port 3000     # 自定义端口
```

浏览器打开 http://127.0.0.1:8080/ 就是 Web 控制台——改配置、切 provider、测试对话，全在网页上操作。REST API：`/health` `/api/config` `/api/providers` `/api/chat`。

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

Prebuilt binaries for 5 platforms are built automatically by GitHub Actions on every tag push — see [Releases](https://github.com/ZSJ305/angles-cli/releases).

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
│   ├── server.rs        # Local HTTP gateway (axum) — angles serve
│   └── tools.rs         # 30+ angles-* tool implementations + doctor
├── instructions.txt     # System prompt template (13K, {{variable}} injection)
├── AGENTS.md            # Agent tool reference
├── providers.toml       # Provider data source
├── gateway-flow.md      # TUI wizard flow spec
├── docs/                # GitHub Pages site (index/docs/tools/faq.html)
├── Cargo.toml
├── Makefile
├── Cross.toml
├── install.sh / install.ps1
└── .github/workflows/release.yml
```

## License

MIT
