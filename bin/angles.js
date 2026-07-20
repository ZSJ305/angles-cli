#!/usr/bin/env node
'use strict';
/**
 * Angles Code CLI — npm launcher
 *
 * - `angles install`  → 运行内置安装脚本 (install.sh / install.ps1) 安装 Rust 二进制
 * - `angles <其他>`    → 转发给已安装的二进制；未安装则提示
 */
const { spawn } = require('child_process');
const path = require('path');
const os = require('os');
const fs = require('fs');

const PKG_ROOT = path.join(__dirname, '..');
const INSTALL_DIR = path.join(os.homedir(), '.local', 'bin');
const BIN_NAME = process.platform === 'win32' ? 'angles.exe' : 'angles';
const BIN_PATH = path.join(INSTALL_DIR, BIN_NAME);
const args = process.argv.slice(2);

function printInstallHint() {
  console.error('angles: 二进制未安装。请先运行:');
  console.error('  angles install');
  console.error('');
  console.error('或手动运行安装脚本:');
  if (process.platform === 'win32') {
    console.error('  irm https://raw.githubusercontent.com/ZSJ305/angles-cli/main/install.ps1 | iex');
  } else {
    console.error('  curl -fsSL https://raw.githubusercontent.com/ZSJ305/angles-cli/main/install.sh | bash');
  }
}

if (args[0] === 'install' || args[0] === 'setup') {
  // ── 子命令：触发安装脚本 ──
  const script = process.platform === 'win32'
    ? path.join(PKG_ROOT, 'install.ps1')
    : path.join(PKG_ROOT, 'install.sh');

  if (!fs.existsSync(script)) {
    console.error('angles: 找不到安装脚本: ' + script);
    process.exit(1);
  }

  console.log('angles: 启动安装器...\n');

  if (process.platform === 'win32') {
    const child = spawn('powershell',
      ['-ExecutionPolicy', 'Bypass', '-File', script],
      { stdio: 'inherit' });
    child.on('exit', (c) => process.exit(c || 0));
  } else {
    const child = spawn('bash', [script], { stdio: 'inherit' });
    child.on('exit', (c) => process.exit(c || 0));
  }
} else {
  // ── 转发给已安装的二进制 ──
  if (!fs.existsSync(BIN_PATH)) {
    printInstallHint();
    process.exit(1);
  }

  const child = spawn(BIN_PATH, args, { stdio: 'inherit' });
  child.on('exit', (c) => process.exit(c || 0));
}
