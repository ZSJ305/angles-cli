use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "angles", version, about = "Angles Code CLI — 终端 agentic 编码助手")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 开始对话（默认模式）
    Chat,

    /// 非交互模式，执行单条指令后退出
    Exec {
        /// 要执行的指令
        prompt: String,
    },

    /// 启动设置向导（TUI 交互式）
    Gateway,

    /// 启动本地 HTTP 网关服务器（Web 控制台 + API）
    Serve {
        /// 端口号（默认 8080）
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },

    /// 显示当前配置
    Config,

    /// 列出所有命令及含义
    Help,

    /// 诊断当前安装和配置
    Doctor,

    /// 查看历史会话
    History,

    /// 恢复历史会话
    Resume {
        /// 会话 ID
        id: String,
    },

    /// 显示/管理当前任务计划
    Plan,

    /// 检查并更新 Angles CLI
    Update,

    /// 管理 Skill（安装、创建、删除）
    Skill {
        #[command(subcommand)]
        action: SkillAction,
    },
}

#[derive(Subcommand)]
pub enum SkillAction {
    /// 列出已安装的 Skill
    List,

    /// 从 GitHub URL 安装 Skill
    Add {
        /// GitHub URL（支持 blob/raw/tree 链接）
        url: String,
    },

    /// 删除已安装的 Skill
    Remove {
        /// Skill 名称
        name: String,
    },

    /// 创建新 Skill（交互式）
    Create,
}

pub fn print_help() {
    println!();
    println!("  α  Angles Code CLI — 终端 agentic 编码助手");
    println!();
    println!("  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("  用户命令:");
    println!();
    println!("  angles           开始对话（默认模式）");
    println!("  angles chat      开始对话");
    println!("  angles exec <p>  非交互模式，执行单条指令");
    println!("  angles gateway   启动设置向导（TUI）");
    println!("  angles config    显示当前配置");
    println!("  angles help      列出所有命令及含义");
    println!("  angles doctor    诊断安装和配置");
    println!("  angles history   查看历史会话");
    println!("  angles resume    恢复历史会话");
    println!("  angles plan      显示/管理当前计划");
    println!("  angles update    检查并更新");
    println!();
    println!("  angles skill list    列出已安装的 Skill");
    println!("  angles skill add     从 GitHub 安装 Skill");
    println!("  angles skill create  创建新 Skill");
    println!("  angles skill remove  删除 Skill");
    println!();
    println!("  angles serve     启动本地 HTTP 网关 (默认 :8080)");
    println!();
    println!("  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("  Agent 工具命令 (angles- 前缀):");
    println!();
    println!("  文件创建:  angles-createfile, angles-writefile,");
    println!("             angles-appendfile, angles-insertline");
    println!();
    println!("  文件读取:  angles-readfile, angles-searchfile,");
    println!("             angles-grep, angles-head, angles-tail");
    println!();
    println!("  文件修改:  angles-replace, angles-replaceall,");
    println!("             angles-deleteline, angles-deletefile,");
    println!("             angles-movedir, angles-copyfile, angles-mkdir");
    println!();
    println!("  目录管理:  angles-ls, angles-tree, angles-pwd,");
    println!("             angles-cd, angles-fileinfo");
    println!();
    println!("  终端执行:  angles-run, angles-runbg, angles-kill");
    println!();
    println!("  网络:      angles-fetch, angles-websearch");
    println!();
    println!("  Git:       angles-gitinit, angles-gitcommit,");
    println!("             angles-gitlog, angles-gitdiff, angles-gitbranch");
    println!();
    println!("  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
}
