mod api;
mod cli;
mod config;
mod gateway;
mod instructions;
mod provider;
mod search;
mod server;
mod skill;
mod tools;

use clap::Parser;

fn main() {
    let args = cli::Cli::parse();

    match &args.command {
        None | Some(cli::Commands::Chat) => {
            let cfg = config::load_or_default();
            if cfg.api_key.is_empty()
                && std::env::var("ANGLES_API_KEY").is_err()
            {
                eprintln!("⚠ API Key 未配置。运行 `angles gateway` 进行设置。");
                std::process::exit(1);
            }
            if let Err(e) = api::start_chat(cfg) {
                eprintln!("❌ 启动失败: {}", e);
                std::process::exit(1);
            }
        }
        Some(cli::Commands::Exec { prompt }) => {
            let cfg = config::load_or_default();
            match api::exec_once(cfg, prompt) {
                Ok(reply) => println!("{}", reply),
                Err(e) => {
                    eprintln!("❌ 执行失败: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(cli::Commands::Gateway) => {
            if let Err(e) = gateway::run_wizard() {
                eprintln!("❌ 设置向导出错: {}", e);
                std::process::exit(1);
            }
        }
        Some(cli::Commands::Config) => {
            let cfg = config::load_or_default();
            config::display(&cfg);
        }
        Some(cli::Commands::Help) => {
            cli::print_help();
        }
        Some(cli::Commands::Doctor) => {
            tools::doctor();
        }
        Some(cli::Commands::History) => {
            let sessions_dir = dirs::home_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join(".angles")
                .join("sessions");
            if sessions_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&sessions_dir) {
                    let mut sessions: Vec<_> = entries.filter_map(|e| e.ok()).collect();
                    sessions.sort_by_key(|e| e.file_name());
                    if sessions.is_empty() {
                        println!("📋 暂无历史会话");
                    } else {
                        println!("📋 历史会话:");
                        for entry in sessions {
                            let name = entry.file_name().to_string_lossy().to_string();
                            println!("  {}", name);
                        }
                    }
                } else {
                    println!("📋 无法读取会话目录");
                }
            } else {
                println!("📋 暂无历史会话目录 ({}尚无会话记录)", sessions_dir.display());
            }
        }
        Some(cli::Commands::Resume { id }) => {
            let _ = id;
            println!("📋 会话恢复功能开发中...");
            println!("   当前的对话模型不支持跨会话持久化，将在 v0.3 实现。");
        }
        Some(cli::Commands::Plan) => {
            println!("📋 当前没有活跃的计划。");
            println!("   计划管理功能将在 v0.3 实现。");
        }
        Some(cli::Commands::Update) => {
            println!("🔄 检查更新...");
            let os = std::env::consts::OS;
            let arch = std::env::consts::ARCH;
            let (os_name, arch_name) = match (os, arch) {
                ("linux", "aarch64") => ("linux", "arm64"),
                ("linux", "x86_64") => ("linux", "x64"),
                ("macos", "aarch64") => ("macos", "arm64"),
                ("macos", "x86_64") => ("macos", "x64"),
                ("windows", "x86_64") => ("windows", "x64"),
                _ => (os, arch),
            };
            let ext = if os == "windows" { "zip" } else { "tar.gz" };
            let url = format!("https://github.com/ZSJ305/angles-cli/releases/latest/download/angles-{}-{}.{}", os_name, arch_name, ext);
            println!("   最新版本: {}", url);
            match std::process::Command::new("curl").args(["-fsSI", "-o", "/dev/null", "-w", "%{http_code}", &url]).output() {
                Ok(o) if String::from_utf8_lossy(&o.stdout).trim() == "200" => {
                    println!("   ✅ 有可用更新!");
                    println!("   运行以下命令更新:");
                    if os == "windows" {
                        println!("     irm https://zsj305.github.io/angles-cli/install.ps1 | iex");
                    } else {
                        println!("     curl -fsSL https://zsj305.github.io/angles-cli/install.sh | bash");
                    }
                }
                _ => println!("   ℹ️  无法检查更新 (网络问题或无新版本)"),
            }
        }
        Some(cli::Commands::Serve { port }) => {
            if let Err(e) = server::start(*port) {
                eprintln!("❌ 网关服务器启动失败: {}", e);
                std::process::exit(1);
            }
        }
        Some(cli::Commands::Skill { action }) => {
            match action {
                cli::SkillAction::List => skill::list(),
                cli::SkillAction::Add { url } => skill::add(url),
                cli::SkillAction::Remove { name } => skill::remove(name),
                cli::SkillAction::Create => skill::create(),
            }
        }
    }
}
