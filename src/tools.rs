use std::process::Command;
use std::fs;
use std::path::Path;

/// Run the doctor diagnostic.
pub fn doctor() {
    println!();
    println!("  🅰  Angles Code CLI — 诊断报告");
    println!();
    println!("  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Binary
    println!("  ✅ angles 二进制: 已安装");
    println!("     架构: {} / {}", std::env::consts::ARCH, std::env::consts::OS);

    println!("  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Binary
    println!("  ✅ angles 二进制: 已安装");

    // Config check
    let cfg = crate::config::load_or_default();
    let cfg_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".angles")
        .join("config.json");
    if cfg_path.exists() {
        println!("  ✅ 配置文件: {}", cfg_path.display());
        println!("     Provider: {}", cfg.provider);
        println!("     Model: {}", cfg.model);
    } else {
        println!("  ⚠️  配置文件: 未找到 (运行 `angles gateway` 创建)");
    }

    // API Key check
    let key_set = !cfg.api_key.is_empty() || std::env::var("ANGLES_API_KEY").is_ok();
    if key_set {
        println!("  ✅ API Key: 已配置");
    } else {
        println!("  ⚠️  API Key: 未配置 (运行 `angles gateway` 设置)");
    }

    // API connectivity test
    if !cfg.base_url.is_empty() {
        let test_url = if cfg.base_url.ends_with("/v1") {
            format!("{}/models", cfg.base_url)
        } else {
            format!("{}/v1/models", cfg.base_url.trim_end_matches('/'))
        };
        let key = if !cfg.api_key.is_empty() { &cfg.api_key }
            else { &std::env::var("ANGLES_API_KEY").unwrap_or_default() };
        match Command::new("curl").args([
            "-s", "-o", "/dev/null", "-w", "%{http_code}",
            "--connect-timeout", "10",
            "-H", &format!("Authorization: Bearer {}", key),
            &test_url,
        ]).output() {
            Ok(o) => {
                let code = String::from_utf8_lossy(&o.stdout).trim().to_string();
                match code.as_str() {
                    "200" => println!("  ✅ API 连通: {} → 200 OK", cfg.base_url),
                    "401" => println!("  ❌ API 连通: {} → 401 (API Key 无效)", cfg.base_url),
                    "404" => println!("  ⚠️  API 连通: {} → 404 (端点不存在)", cfg.base_url),
                    c if c.starts_with("2") => println!("  ✅ API 连通: {} → {}", cfg.base_url, c),
                    c => println!("  ⚠️  API 连通: {} → HTTP {}", cfg.base_url, c),
                }
            }
            _ => println!("  ❌ API 连通: 无法连接 {}", cfg.base_url),
        }
    }

    // Git
    match Command::new("git").arg("--version").output() {
        Ok(o) if o.status.success() => {
            let ver = String::from_utf8_lossy(&o.stdout).trim().to_string();
            println!("  ✅ Git: {}", ver);
        }
        _ => println!("  ⚠️  Git: 未安装"),
    }

    // ripgrep
    match Command::new("rg").arg("--version").output() {
        Ok(o) if o.status.success() => {
            let ver = String::from_utf8_lossy(&o.stdout).lines().next().unwrap_or("").to_string();
            println!("  ✅ ripgrep: {}", ver);
        }
        _ => println!("  ℹ️  ripgrep: 未安装 (angles-grep 将使用 grep 替代)"),
    }

    println!("  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
}

// ─── angles-* tool implementations ───

pub fn angles_createfile(path: &str, content: &str) -> Result<String, String> {
    if Path::new(path).exists() {
        return Err(format!("文件已存在: {}", path));
    }
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    fs::write(path, content).map_err(|e| format!("写入失败: {}", e))?;
    Ok(format!("✅ 已创建: {}", path))
}

pub fn angles_writefile(path: &str, content: &str) -> Result<String, String> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    fs::write(path, content).map_err(|e| format!("写入失败: {}", e))?;
    Ok(format!("✅ 已写入: {}", path))
}

pub fn angles_appendfile(path: &str, content: &str) -> Result<String, String> {
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("打开失败: {}", e))?;
    f.write_all(content.as_bytes()).map_err(|e| format!("追加失败: {}", e))?;
    Ok(format!("✅ 已追加到: {}", path))
}

pub fn angles_readfile(path: &str, start: Option<usize>, end: Option<usize>) -> Result<String, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("读取失败: {}", e))?;
    let lines: Vec<&str> = content.lines().collect();
    let s = start.unwrap_or(1).saturating_sub(1);
    let e = end.unwrap_or(lines.len()).min(lines.len());
    if s >= lines.len() {
        return Ok(String::new());
    }
    Ok(lines[s..e].iter().enumerate()
        .map(|(i, l)| format!("{:>4} | {}", s + i + 1, l))
        .collect::<Vec<_>>()
        .join("\n"))
}

pub fn angles_replace(path: &str, old: &str, new: &str) -> Result<String, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("读取失败: {}", e))?;
    match content.find(old) {
        Some(pos) => {
            let replaced = format!("{}{}{}", &content[..pos], new, &content[pos + old.len()..]);
            fs::write(path, replaced).map_err(|e| format!("写入失败: {}", e))?;
            Ok(format!("✅ 已替换 (1处): {}", path))
        }
        None => Err(format!("未找到匹配文本: {}", old)),
    }
}

pub fn angles_replaceall(path: &str, old: &str, new: &str) -> Result<String, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("读取失败: {}", e))?;
    let count = content.matches(old).count();
    if count == 0 {
        return Err(format!("未找到匹配文本: {}", old));
    }
    let replaced = content.replace(old, new);
    fs::write(path, replaced).map_err(|e| format!("写入失败: {}", e))?;
    Ok(format!("✅ 已替换 ({}处): {}", count, path))
}

pub fn angles_deletefile(path: &str) -> Result<String, String> {
    fs::remove_file(path).map_err(|e| format!("删除失败: {}", e))?;
    Ok(format!("✅ 已删除: {}", path))
}

pub fn angles_mkdir(dir: &str) -> Result<String, String> {
    fs::create_dir_all(dir).map_err(|e| format!("创建失败: {}", e))?;
    Ok(format!("✅ 已创建目录: {}", dir))
}

pub fn angles_movedir(src: &str, dst: &str) -> Result<String, String> {
    fs::rename(src, dst).map_err(|e| format!("移动失败: {}", e))?;
    Ok(format!("✅ 已移动: {} → {}", src, dst))
}

pub fn angles_copyfile(src: &str, dst: &str) -> Result<String, String> {
    fs::copy(src, dst).map_err(|e| format!("复制失败: {}", e))?;
    Ok(format!("✅ 已复制: {} → {}", src, dst))
}

pub fn angles_run(cmd: &str) -> Result<String, String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map_err(|e| format!("执行失败: {}", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let mut result = stdout;
    if !stderr.is_empty() {
        if !result.is_empty() { result.push('\n'); }
        result.push_str(&stderr);
    }
    Ok(result)
}

pub fn angles_searchfile(pattern: &str, directory: &str) -> Result<String, String> {
    let dir = if directory.is_empty() { "." } else { directory };
    let output = Command::new("find")
        .args([dir, "-name", pattern, "-type", "f"])
        .output()
        .map_err(|e| format!("搜索失败: {}", e))?;
    let result = String::from_utf8_lossy(&output.stdout).to_string();
    if result.trim().is_empty() {
        Ok("未找到匹配文件".into())
    } else {
        Ok(result)
    }
}

pub fn angles_grep(pattern: &str, directory: &str) -> Result<String, String> {
    let dir = if directory.is_empty() { "." } else { directory };
    // Try rg first, fall back to grep
    let output = if which::which("rg").is_ok() {
        Command::new("rg").args(["-n", "--no-heading", pattern, dir]).output()
    } else {
        Command::new("grep").args(["-rn", pattern, dir]).output()
    }.map_err(|e| format!("搜索失败: {}", e))?;

    let result = String::from_utf8_lossy(&output.stdout).to_string();
    if result.trim().is_empty() {
        Ok("未找到匹配内容".into())
    } else {
        Ok(result)
    }
}

pub fn angles_websearch(query: &str, engine_url: &str) -> Result<String, String> {
    // For now, return the search URL for the user to open
    // Full scraping would require browser automation or search API
    Ok(format!("🔍 搜索链接: {}\n（完整搜索功能需配合浏览器使用）", engine_url))
}

// ─── Additional tools (补齐到 30+) ───

pub fn angles_appendfile(path: &str, content: &str) -> Result<String, String> {
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("打开失败: {}", e))?;
    f.write_all(content.as_bytes()).map_err(|e| format!("追加失败: {}", e))?;
    Ok(format!("✅ 已追加到: {}", path))
}

pub fn angles_insertline(path: &str, line_num: usize, content: &str) -> Result<String, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("读取失败: {}", e))?;
    let mut lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();
    let idx = line_num.saturating_sub(1).min(lines.len());
    lines.insert(idx, content.to_string());
    fs::write(path, lines.join("\n")).map_err(|e| format!("写入失败: {}", e))?;
    Ok(format!("✅ 已在第 {} 行前插入: {}", line_num, path))
}

pub fn angles_deleteline(path: &str, line_num: usize) -> Result<String, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("读取失败: {}", e))?;
    let mut lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();
    let idx = line_num.saturating_sub(1);
    if idx >= lines.len() {
        return Err(format!("行号超出范围: {} (共 {} 行)", line_num, lines.len()));
    }
    lines.remove(idx);
    fs::write(path, lines.join("\n")).map_err(|e| format!("写入失败: {}", e))?;
    Ok(format!("✅ 已删除第 {} 行: {}", line_num, path))
}

pub fn angles_head(path: &str, n: usize) -> Result<String, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("读取失败: {}", e))?;
    let lines: Vec<&str> = content.lines().take(n).collect();
    Ok(lines.join("\n"))
}

pub fn angles_tail(path: &str, n: usize) -> Result<String, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("读取失败: {}", e))?;
    let lines: Vec<&str> = content.lines().collect();
    let start = lines.len().saturating_sub(n);
    Ok(lines[start..].join("\n"))
}

pub fn angles_ls(dir: &str) -> Result<String, String> {
    let dir = if dir.is_empty() { "." } else { dir };
    let entries = fs::read_dir(dir).map_err(|e| format!("读取目录失败: {}", e))?;
    let mut items: Vec<String> = Vec::new();
    for entry in entries {
        if let Ok(e) = entry {
            let name = e.file_name().to_string_lossy().to_string();
            let ft = e.file_type();
            let prefix = if ft.map(|t| t.is_dir()).unwrap_or(false) { "📁 " } else { "📄 " };
            items.push(format!("{}{}", prefix, name));
        }
    }
    items.sort();
    Ok(items.join("\n"))
}

pub fn angles_tree(dir: &str, depth: usize) -> Result<String, String> {
    let dir = if dir.is_empty() { "." } else { dir };
    fn walk(dir: &Path, prefix: String, depth: usize, max_depth: usize, out: &mut Vec<String>) {
        if depth > max_depth { return; }
        if let Ok(entries) = fs::read_dir(dir) {
            let mut items: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            items.sort_by_key(|e| e.file_name());
            for (i, entry) in items.iter().enumerate() {
                let last = i == items.len() - 1;
                let branch = if last { "└── " } else { "├── " };
                let name = entry.file_name().to_string_lossy().to_string();
                out.push(format!("{}{}{}", prefix, branch, name));
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let next_prefix = if last { format!("{}    ", prefix) } else { format!("{}│   ", prefix) };
                    walk(&entry.path(), next_prefix, depth + 1, max_depth, out);
                }
            }
        }
    }
    let mut out = vec![dir.to_string()];
    walk(Path::new(dir), String::new(), 0, depth, &mut out);
    Ok(out.join("\n"))
}

pub fn angles_pwd() -> Result<String, String> {
    Ok(std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| ".".to_string()))
}

pub fn angles_cd(dir: &str) -> Result<String, String> {
    std::env::set_current_dir(dir).map_err(|e| format!("切换目录失败: {}", e))?;
    Ok(format!("✅ 已切换到: {}", dir))
}

pub fn angles_fileinfo(path: &str) -> Result<String, String> {
    let meta = fs::metadata(path).map_err(|e| format!("获取信息失败: {}", e))?;
    let size = meta.len();
    let perms = if meta.permissions().readonly() { "r--r--r--" } else { "rw-r--r--" };
    let modified = meta.modified()
        .map(|t| {
            let dt: chrono::DateTime<chrono::Local> = t.into();
            dt.format("%Y-%m-%d %H:%M:%S").to_string()
        })
        .unwrap_or_else(|_| "unknown".to_string());
    let ft = if meta.is_dir() { "directory" } else if meta.is_file() { "file" } else { "symlink" };
    Ok(format!("  路径:   {}\n  类型:   {}\n  大小:   {} bytes\n  权限:   {}\n  修改:   {}", path, ft, size, perms, modified))
}

pub fn angles_runbg(cmd: &str) -> Result<String, String> {
    let child = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .spawn()
        .map_err(|e| format!("启动失败: {}", e))?;
    let pid = child.id();
    Ok(format!("✅ 后台启动 (PID={}): {}", pid, cmd))
}

pub fn angles_kill(pid: u32) -> Result<String, String> {
    let pid_str = pid.to_string();
    Command::new("kill")
        .arg(&pid_str)
        .output()
        .map_err(|e| format!("终止失败: {}", e))?;
    Ok(format!("✅ 已发送终止信号: PID={}", pid))
}

pub fn angles_fetch(url: &str, output: &str) -> Result<String, String> {
    let output = Command::new("curl")
        .args(["-fsSL", "-o", output, url])
        .output()
        .map_err(|e| format!("下载失败: {}", e))?;
    if output.status.success() {
        Ok(format!("✅ 已下载: {} → {}", url, output))
    } else {
        Err(format!("下载失败: {}", String::from_utf8_lossy(&output.stderr)))
    }
}

pub fn angles_gitinit(dir: &str) -> Result<String, String> {
    let dir = if dir.is_empty() { "." } else { dir };
    let output = Command::new("git").args(["init", dir]).output()
        .map_err(|e| format!("git init 失败: {}", e))?;
    if output.status.success() {
        Ok(format!("✅ Git 仓库已初始化: {}", dir))
    } else {
        Err(format!("git init 失败: {}", String::from_utf8_lossy(&output.stderr)))
    }
}

pub fn angles_gitcommit(msg: &str) -> Result<String, String> {
    Command::new("git").args(["add", "-A"]).output()
        .map_err(|e| format!("git add 失败: {}", e))?;
    let output = Command::new("git").args(["commit", "-m", msg]).output()
        .map_err(|e| format!("git commit 失败: {}", e))?;
    if output.status.success() {
        Ok(format!("✅ 已提交: {}", msg))
    } else {
        Err(format!("git commit 失败: {}", String::from_utf8_lossy(&output.stderr)))
    }
}

pub fn angles_gitlog(n: usize) -> Result<String, String> {
    let n_str = format!("-{}", n);
    let output = Command::new("git").args(["log", "--oneline", &n_str]).output()
        .map_err(|e| format!("git log 失败: {}", e))?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn angles_gitdiff(path: &str) -> Result<String, String> {
    let mut args = vec!["diff"];
    if !path.is_empty() { args.push(path); }
    let output = Command::new("git").args(&args).output()
        .map_err(|e| format!("git diff 失败: {}", e))?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn angles_gitbranch(name: &str) -> Result<String, String> {
    Command::new("git").args(["checkout", "-b", name]).output()
        .map_err(|e| format!("git branch 失败: {}", e))?;
    Ok(format!("✅ 已创建并切换到分支: {}", name))
}
