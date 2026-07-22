/// Skill management for Angles Code CLI.
///
/// Skills are reusable instruction sets stored at ~/.angles/skills/<name>/SKILL.md.
/// Users can install skills from GitHub URLs, create new ones interactively,
/// list and remove installed skills.
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

/// Get the skills directory: ~/.angles/skills/
fn skills_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".angles")
        .join("skills")
}

/// Ensure the built-in skill-creator is installed.
/// Called automatically on any `angles skill` invocation.
fn ensure_builtin() {
    let dir = skills_dir().join("skill-creator");
    let file = dir.join("SKILL.md");
    if file.exists() {
        return;
    }
    if fs::create_dir_all(&dir).is_err() {
        return;
    }
    let _ = fs::write(&file, BUILTIN_SKILL_CREATOR);
}

/// List all installed skills.
pub fn list() {
    ensure_builtin();
    let dir = skills_dir();
    println!();
    println!("  🅰  Angles Code CLI — 已安装的 Skill");
    println!();
    println!("  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if !dir.exists() {
        println!("  暂无已安装的 Skill");
        println!();
        println!("  安装方式:");
        println!("    angles skill add <github-url>  从 GitHub 安装");
        println!("    angles skill create            创建新 Skill");
        println!();
        return;
    }

    let mut entries: Vec<_> = fs::read_dir(&dir)
        .unwrap_or_else(|_| panic!("无法读取 skills 目录"))
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    if entries.is_empty() {
        println!("  暂无已安装的 Skill");
    } else {
        for entry in entries {
            let name = entry.file_name().to_string_lossy().to_string();
            let skill_md = entry.path().join("SKILL.md");
            let desc = if skill_md.exists() {
                extract_description(&fs::read_to_string(&skill_md).unwrap_or_default())
            } else {
                "(缺少 SKILL.md)".to_string()
            };
            let builtin = if name == "skill-creator" { " [内置]" } else { "" };
            println!("  📦 {:20} {}{}", name, desc, builtin);
        }
    }

    println!();
    println!("  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  目录: {}", dir.display());
    println!();
}

/// Install a skill from a GitHub URL.
/// Accepts:
///   - https://github.com/user/repo/blob/main/path/to/SKILL.md
///   - https://raw.githubusercontent.com/user/repo/main/path/to/SKILL.md
///   - https://github.com/user/repo/tree/main/path/to/skill-dir  (fetches SKILL.md from dir)
pub fn add(url: &str) {
    ensure_builtin();

    let raw_url = github_to_raw(url);
    let content = match fetch_url(&raw_url) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ 下载失败: {}", e);
            eprintln!("   URL: {}", raw_url);
            std::process::exit(1);
        }
    };

    // Extract skill name from frontmatter, fallback to URL path
    let name = extract_name(&content)
        .unwrap_or_else(|| {
            // Try to get from URL path — parent dir of SKILL.md
            let parts: Vec<&str> = raw_url.split('/').collect();
            // raw URL: .../user/repo/branch/path/to/skill-name/SKILL.md
            if parts.len() >= 2 && parts[parts.len() - 1].eq_ignore_ascii_case("skill.md") {
                parts[parts.len() - 2].to_string()
            } else {
                "unnamed-skill".to_string()
            }
        });

    // Validate name — only alphanumeric, hyphens, underscores
    let safe_name: String = name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    if safe_name.is_empty() {
        eprintln!("❌ 无法从 SKILL.md 中解析技能名称（缺少 frontmatter name 字段）");
        std::process::exit(1);
    }

    let skill_dir = skills_dir().join(&safe_name);
    fs::create_dir_all(&skill_dir).expect("无法创建 skill 目录");

    let skill_path = skill_dir.join("SKILL.md");
    fs::write(&skill_path, &content).expect("无法写入 SKILL.md");

    let desc = extract_description(&content);

    println!();
    println!("  ✅ Skill 安装成功！");
    println!();
    println!("  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  名称:   {}", safe_name);
    println!("  描述:   {}", desc);
    println!("  路径:   {}", skill_path.display());
    println!("  来源:   {}", url);
    println!("  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
}

/// Remove a skill by name.
pub fn remove(name: &str) {
    let skill_dir = skills_dir().join(name);
    if !skill_dir.exists() {
        eprintln!("❌ Skill '{}' 不存在", name);
        eprintln!("   运行 `angles skill list` 查看已安装的 Skill");
        std::process::exit(1);
    }

    if name == "skill-creator" {
        eprintln!("⚠️  skill-creator 是内置 Skill，不建议删除。");
        print!("   确认删除? (y/N) ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("  已取消");
            return;
        }
    }

    fs::remove_dir_all(&skill_dir).expect("无法删除 skill 目录");
    println!();
    println!("  🗑️  Skill '{}' 已删除", name);
    println!();
}

/// Create a new skill interactively using the built-in skill-creator.
pub fn create() {
    ensure_builtin();

    println!();
    println!("  🅰  Angles Skill Creator");
    println!();
    println!("  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("  让我们一起创建一个新的 Skill。");
    println!("  Skill 是存放在 ~/.angles/skills/<name>/SKILL.md 的指令集，");
    println!("  让智能体在特定场景下按预定流程工作。");
    println!();

    // Step 1: Name
    print!("  1. Skill 名称 (小写，用连字符，例如 pdf-converter): ");
    io::stdout().flush().unwrap();
    let mut name = String::new();
    io::stdin().read_line(&mut name).unwrap();
    let name = name.trim().to_string();

    if name.is_empty() {
        eprintln!("❌ 名称不能为空");
        return;
    }

    // Validate
    let safe_name: String = name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    if safe_name != name {
        eprintln!("⚠️  名称包含非法字符，已自动清理为: {}", safe_name);
    }

    // Step 2: Description
    print!("  2. 一句话描述这个 Skill 做什么、什么时候触发: ");
    io::stdout().flush().unwrap();
    let mut description = String::new();
    io::stdin().read_line(&mut description).unwrap();
    let description = description.trim().to_string();

    // Step 3: Instructions
    println!();
    println!("  3. 输入 Skill 的核心指令 (输入完按 Enter，然后单独输入 END 结束):");
    println!("     提示:");
    println!("     - 用祈使句，简洁直接");
    println!("     - 只写智能体不知道的知识");
    println!("     - 用例子代替长篇解释");
    println!();
    println!("  ─────────────────────────────────────────────");

    let mut instructions = String::new();
    loop {
        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed == "END" || trimmed == "end" {
                    break;
                }
                instructions.push_str(&line);
            }
            Err(_) => break,
        }
    }

    // Step 4: Generate SKILL.md
    let skill_md = format!(
        "---\nname: {name}\ndescription: {desc}\n---\n\n# {title}\n\n{body}\n",
        name = safe_name,
        desc = description,
        title = safe_name
            .split('-')
            .map(|w| {
                let mut c = w.chars();
                match c.next() {
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
        body = if instructions.is_empty() {
            "Instructions go here.\n".to_string()
        } else {
            instructions
        }
    );

    let skill_dir = skills_dir().join(&safe_name);
    fs::create_dir_all(&skill_dir).expect("无法创建 skill 目录");
    let skill_path = skill_dir.join("SKILL.md");
    fs::write(&skill_path, &skill_md).expect("无法写入 SKILL.md");

    println!();
    println!("  ─────────────────────────────────────────────");
    println!();
    println!("  ✅ Skill 创建成功！");
    println!();
    println!("  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  名称:   {}", safe_name);
    println!("  描述:   {}", description);
    println!("  路径:   {}", skill_path.display());
    println!("  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("  你可以用以下方式分享这个 Skill:");
    println!("    1. 将 SKILL.md 推到 GitHub 仓库");
    println!("    2. 其他人运行: angles skill add <github-url>");
    println!();
    println!("  内置 skill-creator 可以随时帮你创建更多 Skill:");
    println!("    angles skill create");
    println!();
}

/// Ensure the built-in skill-creator is installed (public, for server.rs).
pub fn ensure_builtin_public() {
    ensure_builtin();
}

/// Convert a GitHub URL to a raw URL.
fn github_to_raw(url: &str) -> String {
    // Already raw
    if url.contains("raw.githubusercontent.com") {
        return url.to_string();
    }
    // github.com/user/repo/blob/branch/path → raw.githubusercontent.com/user/repo/branch/path
    if url.contains("github.com") && url.contains("/blob/") {
        return url
            .replace("github.com", "raw.githubusercontent.com")
            .replace("/blob/", "/");
    }
    // github.com/user/repo/tree/branch/path/to/dir → fetch SKILL.md from that dir
    if url.contains("github.com") && url.contains("/tree/") {
        let raw = url
            .replace("github.com", "raw.githubusercontent.com")
            .replace("/tree/", "/");
        // Append SKILL.md if not already there
        if !raw.ends_with("/SKILL.md") && !raw.ends_with("/skill.md") {
            return format!("{}/SKILL.md", raw.trim_end_matches('/'));
        }
        return raw;
    }
    // Fallback: assume it's already a direct URL
    url.to_string()
}

/// Fetch URL content using curl.
fn fetch_url(url: &str) -> Result<String, String> {
    let output = std::process::Command::new("curl")
        .args(["-fsSL", "--connect-timeout", "15", url])
        .output()
        .map_err(|e| format!("无法运行 curl: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("curl 失败: {}", stderr.trim()));
    }

    let content = String::from_utf8_lossy(&output.stdout).to_string();
    if content.is_empty() {
        return Err("下载内容为空".to_string());
    }
    if content.starts_with("404:") || content.contains("Not Found") {
        return Err("文件不存在 (404)".to_string());
    }
    Ok(content)
}

/// Extract `name:` from YAML frontmatter.
fn extract_name(content: &str) -> Option<String> {
    extract_frontmatter_field(content, "name")
}

/// Extract `description:` from YAML frontmatter.
fn extract_description(content: &str) -> String {
    extract_frontmatter_field(content, "description")
        .unwrap_or_else(|| "(无描述)".to_string())
}

/// Generic YAML frontmatter field extractor.
fn extract_frontmatter_field(content: &str, field: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut in_frontmatter = false;

    for line in &lines {
        let trimmed = line.trim();
        if trimmed == "---" {
            in_frontmatter = !in_frontmatter;
            if !in_frontmatter {
                break; // exited frontmatter
            }
            continue;
        }
        if in_frontmatter {
            // Match `field: value` or `field: "value"`
            if let Some(rest) = trimmed.strip_prefix(&format!("{}:", field)) {
                let value = rest.trim();
                // Remove surrounding quotes
                let value = value
                    .trim_matches('"')
                    .trim_matches('\'')
                    .trim()
                    .to_string();
                if !value.is_empty() {
                    return Some(value);
                }
            }
        }
    }
    None
}

/// The built-in skill-creator SKILL.md content.
/// Auto-installed to ~/.angles/skills/skill-creator/SKILL.md on first use.
const BUILTIN_SKILL_CREATOR: &str = r#"---
name: skill-creator
description: Guide for creating effective skills. Use when users want to create a new skill (or update an existing skill) that extends the agent's capabilities with specialized knowledge, workflows, or tool integrations.
---

# Skill Creator

This skill provides guidance for creating effective skills for Angles Code CLI.

## About Skills

Skills are modular, self-contained packages that extend the agent's capabilities.
They are stored at `~/.angles/skills/<name>/SKILL.md` and loaded when triggered.

### What Skills Provide

1. Specialized workflows — multi-step procedures for specific domains
2. Tool integrations — instructions for working with specific file formats or APIs
3. Domain expertise — company-specific knowledge, schemas, business logic

## Core Principles

### Concise is Key

The context window is shared. Only add context the agent doesn't already have.
Prefer concise examples over verbose explanations.

### Set Appropriate Degrees of Freedom

- **High freedom (text-based)**: Use when multiple approaches are valid
- **Medium freedom (pseudocode/params)**: Use when a preferred pattern exists
- **Low freedom (specific scripts)**: Use when operations are fragile or consistency is critical

### Anatomy of a Skill

```
skill-name/
└── SKILL.md (required)
    ├── YAML frontmatter (name + description required)
    └── Markdown instructions
```

#### SKILL.md Frontmatter

- `name` (required): The skill name (lowercase, hyphens)
- `description` (required): What the skill does and when to trigger it — this is the primary triggering mechanism

## Skill Creation Process

1. **Understand** the skill with concrete examples from the user
2. **Plan** reusable contents
3. **Create** the SKILL.md with proper frontmatter and instructions
4. **Test** by using the skill on real tasks
5. **Iterate** based on actual usage

### Writing the SKILL.md

- Use imperative/infinitive form
- `description` field should include all "when to use" triggers
- Only add context the agent doesn't already have
- Prefer concise examples over verbose explanations
- Keep under 500 lines

## Installing Skills

Skills can be installed from GitHub:

```
angles skill add https://github.com/user/repo/blob/main/skills/my-skill/SKILL.md
```

Or created interactively:

```
angles skill create
```

## Sharing Skills

Push your SKILL.md to GitHub and share the URL. Others install with:

```
angles skill add <github-url>
```
"#;
