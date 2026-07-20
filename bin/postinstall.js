#!/usr/bin/env node
'use strict';
/**
 * postinstall 钩子 — 仅打印提示，不强制编译
 * (避免在无 root / 无 Rust 环境下破坏 npm install)
 */
const isWin = process.platform === 'win32';

console.log('');
console.log('  A  Angles Code CLI installed via npm.');
console.log('');
console.log('  Next step — install the angles binary (run once):');
console.log('    angles install');
console.log('');
console.log('  or run the installer script directly:');
if (isWin) {
  console.log('    irm https://raw.githubusercontent.com/ZSJ305/angles-cli/main/install.ps1 | iex');
} else {
  console.log('    curl -fsSL https://raw.githubusercontent.com/ZSJ305/angles-cli/main/install.sh | bash');
}
console.log('');
