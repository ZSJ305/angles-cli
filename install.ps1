#Requires -Version 5.1
# ═══════════════════════════════════════════════════════════════════════
# Angles Code CLI — Installer (PowerShell)
# Usage: irm https://raw.githubusercontent.com/ZSJ305/angles-cli/main/install.ps1 | iex
# ═══════════════════════════════════════════════════════════════════════
$ErrorActionPreference = "Stop"

# ── Colors (ANSI, Win10+ / PS 5.1 via [char]27) ──
$ESC = [char]27
$BOLD        = "$ESC[1m"
$ACCENT      = "$ESC[38;2;90;200;255m"       # angles-blue
$ACCENT_BRIGHT = "$ESC[38;2;130;220;255m"
$INFO        = "$ESC[38;2;136;146;176m"
$SUCCESS     = "$ESC[38;2;0;229;204m"
$WARN        = "$ESC[38;2;255;176;32m"
$ERROR_C     = "$ESC[38;2;230;57;70m"
$MUTED       = "$ESC[38;2;90;100;128m"
$NC          = "$ESC[0m"

# Enable VT100 color support on legacy Windows consoles (best-effort)
try { [Console]::OutputEncoding = [System.Text.Encoding]::UTF8 } catch {}

# ── Config ──
$script:ANGLES_REPO    = if ($env:ANGLES_REPO)    { $env:ANGLES_REPO }    else { "https://github.com/ZSJ305/angles-cli" }
$script:INSTALL_DIR    = if ($env:ANGLES_INSTALL_DIR) { $env:ANGLES_INSTALL_DIR } else { Join-Path $env:USERPROFILE ".local\bin" }
$script:ANGLES_HOME    = if ($env:ANGLES_HOME)    { $env:ANGLES_HOME }    else { Join-Path $env:USERPROFILE ".angles" }
$script:NO_PROMPT      = if ($env:NO_PROMPT)      { [int]$env:NO_PROMPT }  else { 0 }
$script:NO_GATEWAY     = if ($env:NO_GATEWAY)     { [int]$env:NO_GATEWAY } else { 0 }
$script:DRY_RUN        = if ($env:DRY_RUN)        { [int]$env:DRY_RUN }    else { 0 }
$script:HELP           = if ($env:HELP)           { [int]$env:HELP }       else { 0 }
$script:RUST_MIN_VERSION = "1.75.0"

# ── Progress tracking ──
$script:PROGRESS_TOTAL    = 4
$script:PROGRESS_CURRENT  = 0
$script:INSTALL_START_TIME = $null

# ── Temp items cleanup ──
$script:TMPFILES = @()
function Cleanup-TmpFiles {
    foreach ($f in $script:TMPFILES) {
        try { Remove-Item -Recurse -Force $f -ErrorAction SilentlyContinue } catch {}
    }
}
# Note: traps in PS are limited; use try/finally in main instead.

function New-TmpFile {
    $f = [System.IO.Path]::GetTempFileName()
    $script:TMPFILES += $f
    return $f
}

function New-TmpDir {
    $d = Join-Path $env:TEMP ("angles-" + [guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $d -Force | Out-Null
    $script:TMPFILES += $d
    return $d
}

# ── UI helpers ──
# Use ASCII markers for cross-console compatibility (PS 5.1 + legacy codepages)
function ui_info([string]$msg)    { Write-Host "  $INFO$msg$NC" }
function ui_success([string]$msg) { Write-Host "  $SUCCESS[OK] $msg$NC" }
function ui_warn([string]$msg)    { Write-Host "  $WARN[!] $msg$NC" }
function ui_error([string]$msg)   { Write-Host "  $ERROR_C[X] $msg$NC" }
function ui_stage([string]$msg)   { Write-Host ""; Write-Host "  $ACCENT$BOLD[$msg]$NC"; Write-Host "" }
function ui_kv([string]$k, [string]$v) { Write-Host ("  $MUTED{0,-14}$NC {1}" -f $k, $v) }
function ui_section([string]$msg) { Write-Host ""; Write-Host "  $ACCENT$BOLD$msg$NC"; Write-Host "" }

function is_promptable { return ($script:NO_PROMPT -ne 1) -and [Environment]::UserInteractive }

# ═══════════════════════════════════════════════════════════════════════
# Progress Bar
# ═══════════════════════════════════════════════════════════════════════

function progress_advance([string]$stepName) {
    $script:PROGRESS_CURRENT++
    $pct = [math]::Floor($script:PROGRESS_CURRENT * 100 / $script:PROGRESS_TOTAL)
    $elapsed = 0
    if ($script:INSTALL_START_TIME) {
        $elapsed = [math]::Floor((Get-Date).Subtract($script:INSTALL_START_TIME).TotalSeconds)
    }
    $remaining = "--"
    if ($script:PROGRESS_CURRENT -gt 1 -and $elapsed -gt 0) {
        $rate = [math]::Floor($elapsed / $script:PROGRESS_CURRENT)
        $left = $rate * ($script:PROGRESS_TOTAL - $script:PROGRESS_CURRENT)
        if ($left -gt 60) {
            $m = [math]::Floor($left / 60)
            $s = $left % 60
            $remaining = "~${m}分${s}秒"
        } else {
            $remaining = "~${left}秒"
        }
    } elseif ($elapsed -gt 0) {
        $remaining = "计算中..."
    }

    $filled = [math]::Floor($pct * 30 / 100)
    $empty = 30 - $filled
    $bar = ("#" * $filled) + ("-" * $empty)
    Write-Host ""
    Write-Host "  $ACCENT$bar$NC $BOLD$pct%$NC  $MUTED[$($script:PROGRESS_CURRENT)/$($script:PROGRESS_TOTAL)]$NC  $stepName  $MUTED剩余 $remaining$NC"
    Write-Host ""
}

# ── Downloader ──
function download_file([string]$url, [string]$output) {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    Invoke-WebRequest -Uri $url -OutFile $output -UseBasicParsing
}

function test_url([string]$url) {
    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        $r = Invoke-WebRequest -Uri $url -Method Head -UseBasicParsing -ErrorAction Stop
        return $true
    } catch {
        return $false
    }
}

# ═══════════════════════════════════════════════════════════════════════
# Banner
# ═══════════════════════════════════════════════════════════════════════

function print_banner {
    Write-Host ""
    Write-Host "  $ACCENT$BOLD+===========================================+$NC"
    Write-Host "  $ACCENT$BOLD|   A  Angles Code CLI Installer            |$NC"
    Write-Host "  $ACCENT$BOLD|   created by ZSJ                          |$NC"
    Write-Host "  $ACCENT$BOLD+===========================================+$NC"
    Write-Host ""
}

function print_usage {
    Write-Host "Usage:"
    Write-Host "  irm https://raw.githubusercontent.com/ZSJ305/angles-cli/main/install.ps1 | iex"
    Write-Host "  npm i -g @angleschina/angles; angles install"
    Write-Host ""
    Write-Host "Options (via environment):"
    Write-Host "  `$env:NO_PROMPT=1          Skip all interactive prompts"
    Write-Host "  `$env:NO_GATEWAY=1         Skip gateway setup wizard after install"
    Write-Host "  `$env:DRY_RUN=1            Show what would be done without making changes"
    Write-Host "  `$env:ANGLES_REPO='<url>'    Use a custom repository URL"
    Write-Host "  `$env:ANGLES_INSTALL_DIR='<path>'  Custom install directory"
    Write-Host ""
}

# ═══════════════════════════════════════════════════════════════════════
# OS & Architecture Detection (Windows always "windows")
# ═══════════════════════════════════════════════════════════════════════

$script:OS = ""
$script:ARCH = ""
$script:ARCH_LABEL = ""

function detect_os_or_die {
    $script:OS = "windows"
    $raw_arch = $env:PROCESSOR_ARCHITECTURE
    if (-not $raw_arch) { $raw_arch = [Environment]::GetEnvironmentVariable("PROCESSOR_ARCHITECTURE") }
    switch ($raw_arch) {
        "AMD64" { $script:ARCH = "x64";   $script:ARCH_LABEL = "x86_64" }
        "ARM64" { $script:ARCH = "arm64"; $script:ARCH_LABEL = "aarch64" }
        "x86"   { $script:ARCH = "x64";   $script:ARCH_LABEL = "x86_64" }
        default {
            ui_error "不支持的架构: $raw_arch"
            exit 1
        }
    }
    ui_kv "OS" "$script:OS"
    ui_kv "架构" "$script:ARCH_LABEL ($script:ARCH)"
}

# ═══════════════════════════════════════════════════════════════════════
# Build Tools (Windows: winget for Git / VS Build Tools)
# ═══════════════════════════════════════════════════════════════════════

function has_winget {
    $c = Get-Command winget -ErrorAction SilentlyContinue
    return [bool]$c
}

function install_build_tools {
    ui_info "安装编译工具 (Git / VS Build Tools)..."

    # Git via winget
    if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
        if (has_winget) {
            ui_info "通过 winget 安装 Git..."
            winget install --id Git.Git -e --accept-source-agreements --accept-package-agreements --silent 2>&1 | Out-Null
            # Refresh PATH for this session
            $env:PATH = [Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [Environment]::GetEnvironmentVariable("PATH", "User")
        }
    }

    # Visual Studio Build Tools (for Rust MSVC target)
    $hasVS = $false
    $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vswhere) {
        $vsout = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
        if ($vsout) { $hasVS = $true }
    }
    if (-not $hasVS) {
        if (has_winget) {
            ui_info "通过 winget 安装 Visual Studio Build Tools (C++)..."
            ui_warn "这会下载较大的组件，可能需要几分钟。若已装请忽略错误。"
            winget install --id Microsoft.VisualStudio.2022.BuildTools -e --override "--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended" --accept-source-agreements --accept-package-agreements 2>&1 | Out-Null
        } else {
            ui_warn "未检测到 winget，请确保已安装 Git 与 Visual Studio Build Tools (C++)"
        }
    }
}

# ═══════════════════════════════════════════════════════════════════════
# Rust
# ═══════════════════════════════════════════════════════════════════════

function check_rust {
    $c = Get-Command cargo -ErrorAction SilentlyContinue
    if (-not $c) { return $false }
    $verLine = (cargo --version 2>$null)
    if (-not $verLine) { return $false }
    $m = [regex]::Match($verLine, '(\d+)\.(\d+)\.(\d+)')
    if (-not $m.Success) { return $false }
    $major = [int]$m.Groups[1].Value
    $minor = [int]$m.Groups[2].Value
    if ($major -lt 1 -or ($major -eq 1 -and $minor -lt 75)) {
        ui_warn "Rust 版本过低 ($($m.Value) < $script:RUST_MIN_VERSION)，将升级"
        return $false
    }
    return $true
}

function install_rust {
    if (check_rust) {
        ui_success "Rust $((cargo --version 2>$null) -replace '^cargo ','')"
        return
    }
    ui_info "安装 Rust..."
    $tmp = New-TmpFile
    $exe = [System.IO.Path]::ChangeExtension($tmp, ".exe")
    $script:TMPFILES += $exe
    if ($script:ARCH -eq "arm64") {
        download_file "https://win.rustup.rs/aarch64" $exe
    } else {
        download_file "https://win.rustup.rs/x86_64" $exe
    }
    & $exe -y --default-toolchain stable
    # Source cargo env
    $cargoEnv = Join-Path $env:USERPROFILE ".cargo\env.ps1"
    $cargoBat = Join-Path $env:USERPROFILE ".cargo\env"
    if (Test-Path $cargoEnv) { . $cargoEnv }
    # Refresh PATH
    $env:PATH = [Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [Environment]::GetEnvironmentVariable("PATH", "User")
    $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        ui_error "Rust 安装失败"
        exit 1
    }
    ui_success "Rust $((cargo --version 2>$null) -replace '^cargo ','')"
}

# ═══════════════════════════════════════════════════════════════════════
# Git
# ═══════════════════════════════════════════════════════════════════════

function check_git { return [bool](Get-Command git -ErrorAction SilentlyContinue) }

function install_git {
    if (check_git) { return }
    ui_info "安装 Git..."
    if (has_winget) {
        winget install --id Git.Git -e --accept-source-agreements --accept-package-agreements --silent 2>&1 | Out-Null
        $env:PATH = [Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [Environment]::GetEnvironmentVariable("PATH", "User")
    }
    if (-not (check_git)) {
        ui_error "Git 安装失败，请手动安装：https://git-scm.com/download/win"
        exit 1
    }
    ui_success "Git $((git --version 2>$null))"
}

# ═══════════════════════════════════════════════════════════════════════
# Existing Installation Detection
# ═══════════════════════════════════════════════════════════════════════

function check_existing_angles {
    $bin = Join-Path $script:INSTALL_DIR "angles.exe"
    if (Test-Path $bin) {
        $ver = (& $bin --version 2>$null | Select-Object -First 1)
        if ($ver) {
            ui_info "发现已有安装: $ver"
            return $true
        }
    }
    return $false
}

# ═══════════════════════════════════════════════════════════════════════
# Install Angles (pre-built binary or from source)
# ═══════════════════════════════════════════════════════════════════════

function install_angles_binary {
    $binaryName = "angles-$($script:OS)-$($script:ARCH)"
    $url = "$($script:ANGLES_REPO)/releases/latest/download/$binaryName.tar.gz"
    ui_info "检查预编译二进制 $binaryName..."
    if (-not (test_url $url)) { return $false }

    ui_info "下载 $binaryName..."
    $tmp = New-TmpFile
    download_file $url $tmp

    $tmpdir = New-TmpDir
    # Use built-in tar (Win10+)
    & tar xzf $tmp -C $tmpdir 2>$null
    $candidate = Join-Path $tmpdir "angles.exe"
    if (-not (Test-Path $candidate)) {
        $candidate = Join-Path $tmpdir "$binaryName.exe"
    }
    if (-not (Test-Path $candidate)) {
        # maybe .exe without name or angles-exe
        $found = Get-ChildItem $tmpdir -Filter "*.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($found) { $candidate = $found.FullName }
    }
    if (-not (Test-Path $candidate)) {
        ui_warn "预编译二进制解压失败"
        return $false
    }
    if (-not (Test-Path $script:INSTALL_DIR)) { New-Item -ItemType Directory -Path $script:INSTALL_DIR -Force | Out-Null }
    Copy-Item $candidate (Join-Path $script:INSTALL_DIR "angles.exe") -Force
    ui_success "预编译二进制安装完成"
    return $true
}

function install_angles_from_source {
    ui_info "从源码编译..."
    $tmpdir = New-TmpDir
    ui_info "克隆仓库..."
    & git clone --depth 1 $script:ANGLES_REPO (Join-Path $tmpdir "angles-cli")
    Push-Location (Join-Path $tmpdir "angles-cli")

    $cargoArgs = @("build", "--release")
    ui_info "编译中 (cargo $($cargoArgs -join ' '))..."
    & cargo @cargoArgs 2>&1 | Select-Object -Last 5

    $binary = "target\release\angles.exe"
    if (-not (Test-Path $binary)) {
        ui_error "编译失败：找不到 angles 二进制"
        Pop-Location
        exit 1
    }
    if (-not (Test-Path $script:INSTALL_DIR)) { New-Item -ItemType Directory -Path $script:INSTALL_DIR -Force | Out-Null }
    Copy-Item $binary (Join-Path $script:INSTALL_DIR "angles.exe") -Force
    Pop-Location
    ui_success "编译安装完成"
}

# ═══════════════════════════════════════════════════════════════════════
# PATH Setup
# ═══════════════════════════════════════════════════════════════════════

function setup_path {
    $userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    $parts = $userPath -split ";" | Where-Object { $_ -ne "" }
    if ($parts -contains $script:INSTALL_DIR) { return }

    if ($script:DRY_RUN -eq 1) {
        ui_info "Would add $($script:INSTALL_DIR) to User PATH"
        return
    }
    $newPath = ($parts + $script:INSTALL_DIR) -join ";"
    [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
    $env:PATH = "$env:PATH;$($script:INSTALL_DIR)"
    ui_kv "PATH" "Added $($script:INSTALL_DIR) to User PATH"
}

# ═══════════════════════════════════════════════════════════════════════
# Verify Installation
# ═══════════════════════════════════════════════════════════════════════

function verify_installation {
    $anglesBin = Join-Path $script:INSTALL_DIR "angles.exe"
    if (-not (Test-Path $anglesBin)) {
        ui_error "angles 二进制不存在: $anglesBin"
        return $false
    }
    $ver = (& $anglesBin --version 2>$null | Select-Object -First 1)
    if (-not $ver) {
        ui_warn "angles --version 无输出，可能有问题"
        return $false
    }
    ui_success "angles $ver"
    return $true
}

# ═══════════════════════════════════════════════════════════════════════
# Installation Plan
# ═══════════════════════════════════════════════════════════════════════

function show_install_plan {
    Write-Host "  $ACCENT${BOLD}Install plan$NC"
    ui_kv "OS"            $script:OS
    ui_kv "架构"          "$script:ARCH_LABEL ($script:ARCH)"
    ui_kv "安装目录"      $script:INSTALL_DIR
    ui_kv "配置目录"      $script:ANGLES_HOME
    ui_kv "安装方式"      "预编译二进制 (fallback: 源码编译)"
    Write-Host ""
    Write-Host "  $MUTED也可通过 npm 安装：$NC"
    Write-Host "    $ACCENT npm i -g @angleschina/angles; angles install$NC"
    Write-Host ""
}

# ═══════════════════════════════════════════════════════════════════════
# Footer links
# ═══════════════════════════════════════════════════════════════════════

function show_footer {
    Write-Host ""
    $line = "-" * 44
    Write-Host "  $MUTED$line$NC"
    Write-Host "  $MUTED`GitHub:$NC  https://github.com/ZSJ305/angles-cli"
    Write-Host "  $MUTED npm:$NC     npm i -g @angleschina/angles"
    Write-Host "  $MUTED`Docs:$NC    https://github.com/ZSJ305/angles-cli#readme"
    Write-Host "  $MUTED`Report:$NC  https://github.com/ZSJ305/angles-cli/issues"
    Write-Host ""
}

# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════

function main {
    if ($script:HELP -eq 1) { print_usage; return }

    try {
        ui_info "准备安装环境..."
        print_banner
        detect_os_or_die

        $isUpgrade = $false
        if (check_existing_angles) { $isUpgrade = $true }

        show_install_plan

        if ($script:DRY_RUN -eq 1) {
            ui_success "Dry run 完成 (未做任何更改)"
            Cleanup-TmpFiles
            return
        }

        # ── Start timer for progress estimates ──
        $script:INSTALL_START_TIME = Get-Date

        # ── Step 1: Environment (skip build tools if prebuilt available) ──
        progress_advance "准备环境"

        # 先探测预编译二进制是否可用 —— 可用则跳过编译工具 / Rust
        $bn = "angles-$($script:OS)-$($script:ARCH)"
        $cu = "$($script:ANGLES_REPO)/releases/latest/download/$bn.tar.gz"
        $prebuiltAvailable = test_url $cu

        if ($prebuiltAvailable) {
            ui_info "预编译二进制可用，跳过编译工具 / Git / Rust 安装"
        } else {
            ui_info "预编译不可用，将需要从源码编译"
            install_build_tools
            install_git
            install_rust
        }

        # ── Step 2: Install Angles ──
        progress_advance "安装 Angles Code CLI (二进制 / 源码编译)"
        if (-not (Test-Path $script:INSTALL_DIR)) { New-Item -ItemType Directory -Path $script:INSTALL_DIR -Force | Out-Null }
        if (-not (Test-Path $script:ANGLES_HOME)) { New-Item -ItemType Directory -Path $script:ANGLES_HOME -Force | Out-Null }

        if (-not (install_angles_binary)) {
            ui_info "预编译不可用，从源码编译"
            install_angles_from_source
        }

        # ── Step 3: PATH ──
        progress_advance "配置 PATH 环境变量"
        setup_path

        # ── Step 4: Verify & Gateway ──
        progress_advance "验证安装 & 初始配置"
        if (-not (verify_installation)) { exit 1 }

        # ── Success ──
        Write-Host ""
        $anglesBin = Join-Path $script:INSTALL_DIR "angles.exe"
        if ($isUpgrade) {
            $msgs = @(
                "升级完成！新版本已就绪。"
                "焕然一新！更锋利的 A 到手。"
                "代码已更新，bug 已退散。"
                "升级成功！还是那个味，但更快了。"
            )
        } else {
            $msgs = @(
                "终于安家了！准备好大干一场了吗？"
                "安装完成！你的终端从此不一样了。"
                "A 就位！开始写代码吧。"
                "搞定！angles 是你新的编码搭档。"
                "欢迎加入 Angles！让 AI 替你干脏活。"
            )
        }
        $idx = Get-Random -Minimum 0 -Maximum $msgs.Count
        Write-Host "  $MUTED$($msgs[$idx])$NC"
        Write-Host ""

        # Show details
        ui_section "安装详情"
        $ver = (& $anglesBin --version 2>$null | Select-Object -First 1)
        ui_kv "版本"       $ver
        ui_kv "位置"       $anglesBin
        ui_kv "配置"       (Join-Path $script:ANGLES_HOME "config.json")
        ui_kv "升级命令"   "angles update"
        Write-Host ""

        # Run gateway if first install
        if ($script:NO_GATEWAY -ne 1) {
            $cfg = Join-Path $script:ANGLES_HOME "config.json"
            if (-not (Test-Path $cfg)) {
                ui_info "启动设置向导..."
                Write-Host ""
                if (is_promptable) {
                    & $anglesBin gateway
                } else {
                    try {
                        & $anglesBin gateway
                    } catch {
                        ui_warn "设置向导需要交互式终端"
                        Write-Host "  运行以下命令手动配置: angles gateway"
                    }
                }
            } else {
                ui_info "已有配置，跳过向导"
            }
        }

        # ── Final ──
        Write-Host ""
        Write-Host "  $SUCCESS$BOLD[OK] 安装完成!$NC"
        Write-Host ""
        Write-Host "  运行以下命令开始:"
        Write-Host ""
        Write-Host "    $ACCENT`$env:PATH += `";$($script:INSTALL_DIR)`"$NC   # 刷新当前会话 PATH"
        Write-Host "    ${ACCENT}angles$NC              # 开始对话"
        Write-Host ""

        show_footer
    }
    finally {
        Cleanup-TmpFiles
    }
}

# ── Entry point ──
if ($env:ANGLES_INSTALL_PS1_NO_RUN -ne "1") {
    main
}
