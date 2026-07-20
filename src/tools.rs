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

    // Config
    let cfg_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".angles")
        .join("config.json");
    if cfg_path.exists() {
        println!("  ✅ 配置文件: {}", cfg_path.display());
    } else {
        println!("  ⚠️  配置文件: 未找到 (运行 angles-gateway 创建)");
    }

    // Network
    match Command::new("curl").args(["-sI", "--connect-timeout", "5", "https://api.siliconflow.cn/v1/models"]).output() {
        Ok(o) if o.status.success() => println!("  ✅ 网络连通: 正常"),
        _ => println!("  ⚠️  网络连通: 无法访问 API 端点"),
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
