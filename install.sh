#!/usr/bin/env bash
set -euo pipefail

# ═══════════════════════════════════════════════════════════════════════
# Angles Code CLI — Installer
# Usage: curl -fsSL --proto '=https' --tlsv1.2 https://raw.githubusercontent.com/ZSJ305/angles-cli/main/install.sh | bash
# ═══════════════════════════════════════════════════════════════════════

# ── Colors ──
BOLD='\033[1m'
ACCENT='\033[38;2;90;200;255m'       # angles-blue
ACCENT_BRIGHT='\033[38;2;130;220;255m'
INFO='\033[38;2;136;146;176m'
SUCCESS='\033[38;2;0;229;204m'
WARN='\033[38;2;255;176;32m'
ERROR='\033[38;2;230;57;70m'
MUTED='\033[38;2;90;100;128m'
NC='\033[0m'

# ── Config ──
ANGLES_REPO="${ANGLES_REPO:-https://github.com/ZSJ305/angles-cli}"
INSTALL_DIR="${ANGLES_INSTALL_DIR:-$HOME/.local/bin}"
ANGLES_HOME="${ANGLES_HOME:-$HOME/.angles}"
NO_PROMPT="${NO_PROMPT:-0}"
NO_GATEWAY="${NO_GATEWAY:-0}"
DRY_RUN="${DRY_RUN:-0}"
HELP="${HELP:-0}"
RUST_MIN_VERSION="1.75.0"

# ── Temp file cleanup ──
TMPFILES=()
cleanup_tmpfiles() { for f in "${TMPFILES[@]:-}"; do rm -rf "$f" 2>/dev/null || true; done; }
trap cleanup_tmpfiles EXIT
trap 'cleanup_tmpfiles; echo ""; ui_warn "安装中断"; exit 130' INT
trap 'cleanup_tmpfiles; echo ""; ui_warn "安装终止"; exit 143' TERM

mktempfile() { local f; f="$(mktemp)"; TMPFILES+=("$f"); printf -v "$1" '%s' "$f"; }

# ── UI helpers ──
ui_info()    { echo -e "  ${INFO}$1${NC}"; }
ui_success() { echo -e "  ${SUCCESS}✅ $1${NC}"; }
ui_warn()    { echo -e "  ${WARN}⚠️  $1${NC}"; }
ui_error()   { echo -e "  ${ERROR}❌ $1${NC}"; }
ui_stage()   { echo -e "\n  ${ACCENT}${BOLD}[$1]${NC}\n"; }
ui_kv()      { printf "  ${MUTED}%-14s${NC} %s\n" "$1" "$2"; }
ui_section() { echo -e "\n  ${ACCENT}${BOLD}$1${NC}\n"; }

is_promptable() { [[ "$NO_PROMPT" != "1" ]] && [[ -t 0 ]] && [[ -t 1 ]]; }
has_tty()       { [[ -r /dev/tty ]] && [[ -w /dev/tty ]]; }

# ── Downloader ──
DOWNLOADER=""
detect_downloader() {
    if command -v curl &>/dev/null; then DOWNLOADER="curl"; return 0; fi
    if command -v wget &>/dev/null; then DOWNLOADER="wget"; return 0; fi
    ui_error "需要 curl 或 wget"; exit 1
}

download_file() {
    local url="$1" output="$2"
    [[ -z "$DOWNLOADER" ]] && detect_downloader
    if [[ "$DOWNLOADER" == "curl" ]]; then
        curl -fsSL --proto '=https' --tlsv1.2 --speed-limit 1 --speed-time 30 --retry 3 --retry-delay 1 -o "$output" "$url"
    else
        wget -q --https-only --secure-protocol=TLSv1_2 --tries=3 --timeout=20 -O "$output" "$url"
    fi
}

# ═══════════════════════════════════════════════════════════════════════
# Banner
# ═══════════════════════════════════════════════════════════════════════

print_banner() {
    echo ""
    echo -e "  ${ACCENT}${BOLD}╔═══════════════════════════════════════════╗${NC}"
    echo -e "  ${ACCENT}${BOLD}║   🅰  Angles Code CLI Installer       ║${NC}"
    echo -e "  ${ACCENT}${BOLD}║   created by ZSJ🇨🇳                  ║${NC}"
    echo -e "  ${ACCENT}${BOLD}╚═══════════════════════════════════════════╝${NC}"
    echo ""
}

print_usage() {
    echo "Usage: curl -fsSL https://angles.dev/install.sh | bash [options]"
    echo ""
    echo "Options (via environment):"
    echo "  NO_PROMPT=1          Skip all interactive prompts"
    echo "  NO_GATEWAY=1         Skip gateway setup wizard after install"
    echo "  DRY_RUN=1            Show what would be done without making changes"
    echo "  ANGLES_REPO=<url>    Use a custom repository URL"
    echo "  ANGLES_INSTALL_DIR=<path>  Custom install directory"
    echo ""
}

# ═══════════════════════════════════════════════════════════════════════
# OS & Architecture Detection
# ═══════════════════════════════════════════════════════════════════════

OS=""
ARCH=""
ARCH_LABEL=""

detect_os_or_die() {
    OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
    case "$OS" in
        linux)  OS="linux" ;;
        darwin) OS="macos" ;;
        msys*|mingw*|cygwin*) OS="windows" ;;
        *)      ui_error "不支持的操作系统: $(uname -s)"; exit 1 ;;
    esac

    local raw_arch
    raw_arch="$(uname -m)"
    case "$raw_arch" in
        x86_64|amd64)   ARCH="x64";    ARCH_LABEL="x86_64" ;;
        aarch64|arm64)  ARCH="arm64";  ARCH_LABEL="aarch64" ;;
        armv7l|armv7)   ARCH="armv7";  ARCH_LABEL="armv7" ;;
        riscv64)        ARCH="riscv64"; ARCH_LABEL="riscv64" ;;
        *)              ui_error "不支持的架构: $raw_arch"; exit 1 ;;
    esac

    ui_kv "OS" "$OS"
    ui_kv "架构" "$ARCH_LABEL ($ARCH)"
}

# ═══════════════════════════════════════════════════════════════════════
# Build Tools
# ═══════════════════════════════════════════════════════════════════════

install_build_tools_linux() {
    ui_info "安装编译工具..."
    if command -v apt-get &>/dev/null; then
        apt-get update -qq
        apt-get install -y -qq build-essential pkg-config libssl-dev curl git 2>/dev/null
    elif command -v apk &>/dev/null; then
        apk add --no-cache build-base openssl-dev curl git
    elif command -v dnf &>/dev/null; then
        dnf install -y gcc gcc-c++ make openssl-devel curl git 2>/dev/null
    elif command -v yum &>/dev/null; then
        yum install -y gcc gcc-c++ make openssl-devel curl git 2>/dev/null
    elif command -v pacman &>/dev/null; then
        pacman -Sy --noconfirm base-devel openssl curl git 2>/dev/null
    elif command -v zypper &>/dev/null; then
        zypper install -y gcc gcc-c++ make libopenssl-devel curl git 2>/dev/null
    elif command -v emerge &>/dev/null; then
        emerge --ask=n sys-devel/gcc dev-libs/openssl net-misc/curl dev-vcs/git 2>/dev/null
    elif command -v xbps-install &>/dev/null; then
        xbps-install -Sy base-devel openssl curl git 2>/dev/null
    else
        ui_warn "未识别的包管理器，请确保已安装: gcc, openssl-dev, curl, git"
        return 1
    fi
}

install_build_tools_macos() {
    if ! xcode-select -p &>/dev/null; then
        ui_info "安装 Xcode Command Line Tools..."
        xcode-select --install 2>/dev/null || true
        ui_info "等待安装完成..."
        while ! xcode-select -p &>/dev/null; do sleep 5; done
    fi
    ui_success "Xcode CLT 已就绪"
}

install_build_tools() {
    case "$OS" in
        linux)  install_build_tools_linux ;;
        macos)  install_build_tools_macos ;;
        windows) ui_warn "Windows 建议使用 WSL2" ;;
    esac
}

# ═══════════════════════════════════════════════════════════════════════
# Rust
# ═══════════════════════════════════════════════════════════════════════

check_rust() {
    if ! command -v cargo &>/dev/null; then
        return 1
    fi
    local ver
    ver="$(cargo --version 2>/dev/null | grep -oP '\d+\.\d+\.\d+' | head -1)"
    if [[ -z "$ver" ]]; then return 1; fi
    # Simple major.minor comparison
    local major minor
    IFS='.' read -r major minor _ <<< "$ver"
    if [[ "$major" -lt 1 ]] || { [[ "$major" -eq 1 ]] && [[ "$minor" -lt 75 ]]; }; then
        ui_warn "Rust $ver 版本过低 (需要 >= $RUST_MIN_VERSION)，将升级"
        return 1
    fi
    return 0
}

install_rust() {
    if check_rust; then
        ui_success "Rust $(cargo --version | head -1 | grep -oP '\d+\.\d+\.\d+')"
        return 0
    fi

    ui_info "安装 Rust..."
    local tmp; mktempfile tmp
    download_file "https://sh.rustup.rs" "$tmp"

    if is_promptable; then
        if has_tty; then
            /bin/bash "$tmp" -y --default-toolchain stable < /dev/tty
        else
            /bin/bash "$tmp" -y --default-toolchain stable
        fi
    else
        /bin/bash "$tmp" -y --default-toolchain stable < /dev/null
    fi

    # Source cargo env
    if [[ -f "$HOME/.cargo/env" ]]; then
        source "$HOME/.cargo/env"
    fi

    if ! command -v cargo &>/dev/null; then
        ui_error "Rust 安装失败"
        exit 1
    fi

    ui_success "Rust $(cargo --version | head -1)"
}

# ═══════════════════════════════════════════════════════════════════════
# Git
# ═══════════════════════════════════════════════════════════════════════

check_git() {
    command -v git &>/dev/null
}

install_git() {
    if check_git; then return 0; fi
    ui_info "安装 Git..."
    if [[ "$OS" == "linux" ]]; then
        if command -v apt-get &>/dev/null; then
            apt-get install -y -qq git
        elif command -v apk &>/dev/null; then
            apk add --no-cache git
        elif command -v dnf &>/dev/null; then
            dnf install -y git
        elif command -v pacman &>/dev/null; then
            pacman -Sy --noconfirm git
        fi
    fi
    if ! check_git; then
        ui_error "Git 安装失败，请手动安装"
        exit 1
    fi
    ui_success "Git $(git --version)"
}

# ═══════════════════════════════════════════════════════════════════════
# Existing Installation Detection
# ═══════════════════════════════════════════════════════════════════════

check_existing_angles() {
    if [[ -x "$INSTALL_DIR/angles" ]]; then
        local ver
        ver="$("$INSTALL_DIR/angles" --version 2>/dev/null | head -1 || true)"
        if [[ -n "$ver" ]]; then
            ui_info "发现已有安装: $ver"
            return 0
        fi
    fi
    return 1
}

# ═══════════════════════════════════════════════════════════════════════
# Install Angles (pre-built binary or from source)
# ═══════════════════════════════════════════════════════════════════════

install_angles_binary() {
    local binary_name="angles-${OS}-${ARCH}"
    local url="${ANGLES_REPO}/releases/latest/download/${binary_name}.tar.gz"

    ui_info "检查预编译二进制 ${binary_name}..."

    # Check if binary exists at URL
    if ! curl -fsSL --connect-timeout 10 --head "$url" 2>/dev/null; then
        return 1
    fi

    ui_info "下载 ${binary_name}..."
    local tmp; mktempfile tmp
    download_file "$url" "$tmp"
    tar xzf "$tmp" -C "$INSTALL_DIR" angles 2>/dev/null || {
        # Maybe the tar has a different structure
        local tmpdir; tmpdir="$(mktemp -d)"; TMPFILES+=("$tmpdir")
        tar xzf "$tmp" -C "$tmpdir"
        cp "$tmpdir/angles" "$INSTALL_DIR/angles" 2>/dev/null || \
        cp "$tmpdir/${binary_name}" "$INSTALL_DIR/angles" 2>/dev/null || {
            ui_warn "预编译二进制解压失败"
            return 1
        }
    }

    chmod +x "$INSTALL_DIR/angles"
    ui_success "预编译二进制安装完成"
    return 0
}

install_angles_from_source() {
    ui_info "从源码编译..."

    local tmpdir; tmpdir="$(mktemp -d)"; TMPFILES+=("$tmpdir")
    local repo_url="${ANGLES_REPO}"

    ui_info "克隆仓库..."
    git clone --depth 1 "$repo_url" "$tmpdir/angles-cli"

    cd "$tmpdir/angles-cli"

    # Choose target for static linking on Linux
    local cargo_args="build --release"
    if [[ "$OS" == "linux" ]]; then
        if [[ "$ARCH" == "arm64" ]]; then
            rustup target add aarch64-unknown-linux-musl 2>/dev/null || true
            if command -v aarch64-linux-musl-gcc &>/dev/null; then
                cargo_args="build --release --target aarch64-unknown-linux-musl"
            fi
        elif [[ "$ARCH" == "x64" ]]; then
            rustup target add x86_64-unknown-linux-musl 2>/dev/null || true
            if command -v x86_64-linux-musl-gcc &>/dev/null; then
                cargo_args="build --release --target x86_64-unknown-linux-musl"
            fi
        fi
    fi

    ui_info "编译中 (cargo $cargo_args)..."
    cargo $cargo_args 2>&1 | tail -5

    # Find the binary
    local binary=""
    for candidate in \
        "target/aarch64-unknown-linux-musl/release/angles" \
        "target/x86_64-unknown-linux-musl/release/angles" \
        "target/release/angles"; do
        if [[ -f "$candidate" ]]; then
            binary="$candidate"
            break
        fi
    done

    if [[ -z "$binary" ]]; then
        ui_error "编译失败：找不到 angles 二进制"
        exit 1
    fi

    cp "$binary" "$INSTALL_DIR/angles"
    chmod +x "$INSTALL_DIR/angles"
    cd - > /dev/null

    ui_success "编译安装完成"
}

# ═══════════════════════════════════════════════════════════════════════
# PATH Setup
# ═══════════════════════════════════════════════════════════════════════

setup_path() {
    local rc=""
    case "$SHELL" in
        */zsh)  rc="$HOME/.zshrc" ;;
        */bash) rc="$HOME/.bashrc" ;;
        */fish) rc="$HOME/.config/fish/config.fish" ;;
        */elvish) rc="$HOME/.config/elvish/rc.elv" ;;
        */nu) rc="$HOME/.config/nushell/config.nu" ;;
        *)      rc="$HOME/.profile" ;;
    esac

    # Check if already in PATH
    if [[ ":$PATH:" == *":$INSTALL_DIR:"* ]]; then
        return 0
    fi

    if [[ "$DRY_RUN" == "1" ]]; then
        ui_info "Would add $INSTALL_DIR to PATH in $rc"
        return 0
    fi

    if [[ "$SHELL" == */fish ]]; then
        echo "fish_add_path $INSTALL_DIR" >> "$rc"
    elif [[ "$SHELL" == */nu ]]; then
        echo "let-env PATH = (\$env.PATH | append '$INSTALL_DIR')" >> "$rc"
    else
        echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$rc"
    fi

    export PATH="$PATH:$INSTALL_DIR"
    ui_kv "PATH" "Added $INSTALL_DIR to $rc"

    # Symlink to /usr/local/bin if writable
    if [[ -w /usr/local/bin ]] && [[ ! -e /usr/local/bin/angles ]]; then
        ln -sf "$INSTALL_DIR/angles" /usr/local/bin/angles 2>/dev/null && \
            ui_kv "Symlink" "/usr/local/bin/angles → $INSTALL_DIR/angles"
    fi
}

# ═══════════════════════════════════════════════════════════════════════
# Verify Installation
# ═══════════════════════════════════════════════════════════════════════

verify_installation() {
    local angles_bin="$INSTALL_DIR/angles"
    if [[ ! -x "$angles_bin" ]]; then
        ui_error "angles 二进制不存在: $angles_bin"
        return 1
    fi

    local ver
    ver="$("$angles_bin" --version 2>/dev/null | head -1 || true)"
    if [[ -z "$ver" ]]; then
        ui_warn "angles --version 无输出，可能有问题"
        return 1
    fi

    ui_success "angles $ver"
    return 0
}

# ═══════════════════════════════════════════════════════════════════════
# Installation Plan
# ═══════════════════════════════════════════════════════════════════════

show_install_plan() {
    echo -e "  ${ACCENT}${BOLD}Install plan${NC}"
    ui_kv "OS"            "$OS"
    ui_kv "架构"          "$ARCH_LABEL ($ARCH)"
    ui_kv "安装目录"      "$INSTALL_DIR"
    ui_kv "配置目录"      "$ANGLES_HOME"
    ui_kv "安装方式"      "预编译二进制 (fallback: 源码编译)"
    echo ""
}

# ═══════════════════════════════════════════════════════════════════════
# Footer links
# ═══════════════════════════════════════════════════════════════════════

show_footer() {
    echo ""
    echo -e "  ${MUTED}━$(printf '━%.0s' {1..43})━${NC}"
    echo -e "  ${MUTED}GitHub:${NC}  https://github.com/ZSJ305/angles-cli"
    echo -e "  ${MUTED}Docs:${NC}    https://github.com/ZSJ305/angles-cli#readme"
    echo -e "  ${MUTED}Report:${NC}  https://github.com/ZSJ305/angles-cli/issues"
    echo ""
}

# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════

main() {
    if [[ "$HELP" == "1" ]]; then print_usage; return 0; fi

# ── Root check ──
    if [[ "$(id -u)" -ne 0 ]]; then
        echo ""
        ui_error "请以 root 用户运行此脚本"
        echo -e "  ${MUTED}sudo curl ... | sudo bash${NC}"
        echo -e "  ${MUTED}或: su -c 'bash install.sh'${NC}"
        echo ""
        exit 1
    fi

    ui_info "准备安装环境..."
    detect_downloader
    print_banner
    detect_os_or_die

    local is_upgrade=false
    if check_existing_angles; then
        is_upgrade=true
    fi

    show_install_plan

    if [[ "$DRY_RUN" == "1" ]]; then
        ui_success "Dry run 完成 (未做任何更改)"
        return 0
    fi

    # ── Step 1: Environment ──
    ui_stage "[1/4] 准备环境"

    if [[ "$OS" == "linux" ]]; then
        export DEBIAN_FRONTEND="${DEBIAN_FRONTEND:-noninteractive}"
        export NEEDRESTART_MODE="${NEEDRESTART_MODE:-a}"
    fi

    install_build_tools
    install_git
    install_rust

    # ── Step 2: Install Angles ──
    ui_stage "[2/4] 安装 Angles Code CLI"

    mkdir -p "$INSTALL_DIR"
    mkdir -p "$ANGLES_HOME"

    if ! install_angles_binary; then
        ui_info "预编译不可用，从源码编译"
        install_angles_from_source
    fi

    # ── Step 3: PATH ──
    ui_stage "[3/4] 配置 PATH"
    setup_path

    # ── Step 4: Verify & Gateway ──
    ui_stage "[4/4] 验证 & 初始配置"

    if ! verify_installation; then
        exit 1
    fi

    # ── Success ──
    echo ""
    if [[ "$is_upgrade" == "true" ]]; then
        local upgrade_msgs=(
            "升级完成！新版本已就绪。"
            "焕然一新！更锋利的 🅰 到手。"
            "代码已更新，bug 已退散。"
            "升级成功！还是那个味，但更快了。"
        )
        echo -e "  ${MUTED}${upgrade_msgs[$((RANDOM % ${#upgrade_msgs[@]}))]}${NC}"
    else
        local install_msgs=(
            "终于安家了！准备好大干一场了吗？"
            "安装完成！你的终端从此不一样了。"
            "🅰 就位！开始写代码吧。"
            "搞定！angles 是你新的编码搭档。"
            "欢迎加入 Angles！让 AI 替你干脏活。"
        )
        echo -e "  ${MUTED}${install_msgs[$((RANDOM % ${#install_msgs[@]}))]}${NC}"
    fi
    echo ""

    # Show details
    ui_section "安装详情"
    ui_kv "版本"       "$("$INSTALL_DIR/angles" --version 2>/dev/null | head -1)"
    ui_kv "位置"       "$INSTALL_DIR/angles"
    ui_kv "配置"       "$ANGLES_HOME/config.json"
    ui_kv "升级命令"   "angles update"
    echo ""

    # Run gateway if first install
    if [[ "$NO_GATEWAY" != "1" ]]; then
        if [[ ! -f "$ANGLES_HOME/config.json" ]]; then
            ui_info "启动设置向导..."
            echo ""
            if is_promptable && has_tty; then
                "$INSTALL_DIR/angles" gateway < /dev/tty
            else
                "$INSTALL_DIR/angles" gateway || {
                    ui_warn "设置向导需要交互式终端"
                    echo "  运行以下命令手动配置: angles gateway"
                }
            fi
        else
            ui_info "已有配置，跳过向导"
        fi
    fi

    # ── Final ──
    echo ""
    echo -e "  ${SUCCESS}${BOLD}✅ 安装完成!${NC}"
    echo ""
    echo "  运行以下命令开始:"
    echo ""
    echo -e "    ${ACCENT}source ~/.bashrc${NC}   # 或 ~/.zshrc"
    echo -e "    ${ACCENT}angles${NC}              # 开始对话"
    echo ""

    show_footer
}

# ── Entry point ──
if [[ "${ANGLES_INSTALL_SH_NO_RUN:-0}" != "1" ]]; then
    main
fi
