# Angles Gateway — TUI Setup Wizard Flow

## 启动
```
$ angles-gateway
```

 显示 Angles Logo 并进入交互式向导。

---

## Step 1: 语言选择

```
┌─────────────────────────────────────┐
│       🅰 Angles Code CLI Setup      │
│                                     │
│  请选择语言 / Select Language:       │
│                                     │
│  > 中文 (zh-CN)                     │
│    English (en-US)                  │
│    日本語 (ja-JP)                   │
│                                     │
│  ↑↓ 选择  Enter 确认               │
└─────────────────────────────────────┘
```

选择后所有后续界面和 agent 默认语言随之切换。

---

## Step 2: 模型提供方

```
┌─────────────────────────────────────┐
│  🅰 选择模型提供方                    │
│                                     │
│  > OpenAI        api.openai.com     │
│    Claude        api.anthropic.com  │
│    Gemini        googleapis.com     │
│    DeepSeek      api.deepseek.com   │
│    Grok          api.x.ai           │
│    MiniMax       api.minimax.chat   │
│    OpenRouter    openrouter.ai      │
│    通义千问 Qwen  dashscope.aliyuncs │
│    智谱 GLM      api.siliconflow.cn │
│    Kimi          api.moonshot.cn    │
│    自定义 Custom  (手动填写)          │
│                                     │
│  ↑↓ 选择  Enter 确认               │
└─────────────────────────────────────┘
```

- 选中非自定义提供方：自动填充 base_url，进入 Step 2a
- 选中自定义：进入 Step 2b

### Step 2a: API Key 输入（已知提供方）

```
┌─────────────────────────────────────┐
│  🅰 OpenAI — 输入 API Key           │
│                                     │
│  API Host: https://api.openai.com/v1│
│                                     │
│  API Key: sk-****__________________ │
│                                     │
│  (留空跳过，之后通过环境变量配置)      │
│                                     │
│  Enter 确认  Esc 返回               │
└─────────────────────────────────────┘
```

- Key 以 `sk-` 或对应前缀开头
- 输入时自动隐藏（显示为 `*`）
- 可留空跳过
- 确认后可选"测试连接"验证 key 是否有效

### Step 2b: 自定义提供方

```
┌─────────────────────────────────────┐
│  🅰 自定义提供方                      │
│                                     │
│  API Base URL: https://___________  │
│  API Key:      sk-_______________   │
│  模型 ID:      __________________   │
│  协议:         [OpenAI Chat ✓]      │
│                [Anthropic Messages]  │
│                [Gemini Native]       │
│                                     │
│  Enter 确认  Esc 返回               │
└─────────────────────────────────────┘
```

---

## Step 3: 模型选择

```
┌─────────────────────────────────────┐
│  🅰 选择默认模型                      │
│                                     │
│  Provider: OpenAI                   │
│                                     │
│  > gpt-4.1         (推荐)           │
│    gpt-4.1-mini                     │
│    gpt-4.1-nano                     │
│    o3                               │
│    o3-mini                          │
│    o4-mini                          │
│    手动输入模型 ID                    │
│                                     │
│  ↑↓ 选择  Enter 确认               │
└─────────────────────────────────────┘
```

- 列表从 providers.toml 读取
- 最后一个选项允许手动输入任意模型 ID

---

## Step 4: 偏好设置

```
┌─────────────────────────────────────┐
│  🅰 偏好设置                         │
│                                     │
│  最大输出 Token: [16384      ]      │
│  每日 Token 预算: [1000000   ]      │
│                                     │
│  Agent 人设:                        │
│  ┌─────────────────────────────┐    │
│  │你是一个专业、高效的编码助手。 │    │
│  │擅长系统编程和快速排错。      │    │
│  │回答简洁直接，不废话。        │    │
│  └─────────────────────────────┘    │
│  (几句话描述你希望 agent 的行为)     │
│                                     │
│  审批策略:                          │
│  > untrusted  (安全命令自动执行)     │
│    on-request (agent决定何时问)      │
│    never      (全部自动执行)         │
│                                     │
│  Enter 确认  Esc 返回               │
└─────────────────────────────────────┘
```

- **最大 Token**: 每次回复的最大 token 数，默认 16384
- **每日预算**: 每天最多消耗的 token 总量，默认 1000000
- **Agent 人设**: 自由文本，2-3 句话描述期望行为，会注入 instructions 的 `{{agent_persona}}`
- **审批策略**: 控制工具调用是否需要用户确认

---

## Step 5: 联网搜索

```
┌─────────────────────────────────────┐
│  🅰 联网搜索配置                      │
│                                     │
│  首选搜索引擎:                       │
│                                     │
│  > Bing       (综合 + AI 摘要)      │
│    Baidu       (中文内容丰富)        │
│    Google      (覆盖最广)            │
│    Yahoo       (备用)               │
│    自定义 URL  (自选搜索引擎)         │
│    关闭        (不使用联网搜索)       │
│                                     │
│  ↑↓ 选择  Enter 确认               │
└─────────────────────────────────────┘
```

选"自定义 URL"时：

```
┌─────────────────────────────────────┐
│  🅰 自定义搜索引擎                    │
│                                     │
│  搜索 URL 模板:                     │
│  https://search.example.com/?q={q}  │
│                                     │
│  {q} 会被替换为搜索关键词            │
│                                     │
│  Enter 确认  Esc 返回               │
└─────────────────────────────────────┘
```

---

## 完成界面

```
┌─────────────────────────────────────┐
│  ✅ Angles Code CLI 配置完成!        │
│                                     │
│  语言:     中文 (zh-CN)              │
│  提供方:   智谱 GLM                  │
│  模型:     zai-org/GLM-5.2          │
│  最大Token: 16384                    │
│  日预算:   1,000,000                 │
│  搜索引擎: Bing                      │
│  审批策略: untrusted                 │
│                                     │
│  配置已保存到 ~/.angles/config.json  │
│                                     │
│  运行 angles-chat 开始对话! 🚀       │
│                                     │
│  Enter 退出                         │
└─────────────────────────────────────┘
```

---

## 配置文件格式

保存在 `~/.angles/config.json`：

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
  "approval_policy": "untrusted",
  "daily_tokens_used": 0,
  "daily_reset_date": "2026-07-20"
}
```

- `api_key`: 留空时从环境变量 `ANGLES_API_KEY` 读取
- `wire_api`: `chat` (OpenAI 兼容) | `anthropic` | `gemini`
- `daily_tokens_used`: 当日已消耗 token 数
- `daily_reset_date`: 每日重置日期
