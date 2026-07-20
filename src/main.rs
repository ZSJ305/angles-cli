mod api;
mod cli;
mod config;
mod gateway;
mod instructions;
mod provider;
mod search;
mod server;
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
            if let Err(e) = api::exec_once(cfg, prompt) {
                eprintln!("❌ 执行失败: {}", e);
                std::process::exit(1);
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
            println!("📋 历史会话功能开发中...");
        }
        Some(cli::Commands::Resume { id: _ }) => {
            println!("📋 恢复会话功能开发中...");
        }
        Some(cli::Commands::Plan) => {
            println!("📋 计划管理功能开发中...");
        }
        Some(cli::Commands::Update) => {
            println!("🔄 检查更新...");
        }
    }
}
