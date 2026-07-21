#!/usr/bin/env python3
"""Render a high-fidelity 'real angles terminal screenshot' PNG from lines of
colored text. macOS-style window chrome + JetBrains Mono on warm-black bg."""

from PIL import Image, ImageDraw, ImageFont

# ── Canvas config ──
W = 1180
PAD_X = 36
PAD_TOP = 56          # below title bar
PAD_BOTTOM = 36
LINE_H = 30
FONT_SIZE = 19

BG = (28, 26, 18)     # #1c1a12 warm-black
BAR_BG = (40, 37, 28) # #28251c
TITLE_COL = (138, 132, 116)

FONT = "/usr/share/fonts/jetbrains-mono/JetBrainsMono-Regular.ttf"
FONT_BOLD = "/usr/share/fonts/jetbrains-mono/JetBrainsMono-SemiBold.ttf"
FONT_ITAL = "/usr/share/fonts/jetbrains-mono/JetBrainsMono-Italic.ttf"

font = ImageFont.truetype(FONT, FONT_SIZE)
font_bold = ImageFont.truetype(FONT_BOLD, FONT_SIZE)

# Color palette (matching the HTML mockup)
C = {
    "p": (201, 195, 176),  # prompt  #c9c3b0
    "c": (107, 102, 87),   # comment #6b6657
    "o": (168, 224, 160),  # ok      #a8e0a0
    "b": (130, 177, 255),  # blue    #82b1ff
    "g": (255, 212, 121),  # gold    #ffd479
    "m": (214, 168, 255),  # magenta #d6a8ff
    "w": (241, 237, 226),  # text    #f1ede2
}

# Each line: list of (text, color_key) tuples
lines = [
    [("~/my-app", "p"), (" $ ", "c"), ("angles", "o"), (" chat", "w")],
    [("", None)],
    [("  α  ", "o"), ("Angles Code CLI v0.2.1", "w"), ("  ·  11 models  ·  30 tools", "c")],
    [("  ─────────────────────────────────────────────────────────", "c")],
    [("", None)],
    [("You", "b"), ("   给这个 Express 项目加 JWT 鉴权，写测试", "w")],
    [("", None)],
    [("α", "o"), ("   正在规划...", "c")],
    [("  ▸ 读取 ", "c"), ("package.json", "o"), (" 和 ", "c"), ("src/", "o"), (" 结构", "c")],
    [("  ▸ 安装 ", "c"), ("jsonwebtoken", "o"), (" + ", "c"), ("bcryptjs", "o")],
    [("  ▸ 创建 ", "c"), ("src/middleware/auth.ts", "o")],
    [("  ▸ 创建 ", "c"), ("tests/auth.test.ts", "o")],
    [("  ▸ 运行 ", "c"), ("npm test", "g")],
    [("  ▸ git commit ", "c"), ("-m \"feat: JWT auth\"", "c")],
    [("", None)],
    [("α", "o"), ("   ✓ 完成 · 4 文件已修改 · 1 文件已创建", "o")],
    [("", None)],
    [("α", "o"), ("   运行测试...", "c")],
    [("    ✓ ", "o"), ("auth.test.ts", "o"), ("  (8 passed)", "c")],
    [("", None)],
    [("~/my-app", "p"), (" $ ", "c"), ("_", "o")],
]

# ── Compute size ──
H = PAD_TOP + len(lines) * LINE_H + PAD_BOTTOM

img = Image.new("RGB", (W, H), BG)
draw = ImageDraw.Draw(img)

# ── Title bar ──
draw.rectangle([(0, 0), (W, 42)], fill=BAR_BG)
# Traffic-light dots
dot_y = 21
for i, color in enumerate([(255, 95, 86), (255, 189, 46), (39, 201, 63)]):
    cx = 22 + i * 22
    draw.ellipse([(cx - 7, dot_y - 7), (cx + 7, dot_y + 7)], fill=color)
# Title (centered)
title = "angles — zsh — 132×40"
t_w = draw.textlength(title, font=font)
draw.text(((W - t_w) / 2, 11), title, font=font, fill=TITLE_COL)
# Subtle divider
draw.line([(0, 42), (W, 42)], fill=(58, 54, 42))

# ── Lines ──
y = PAD_TOP
for parts in lines:
    x = PAD_X
    for text, key in parts:
        if not text:
            continue
        col = C.get(key, C["w"]) if key else C["w"]
        draw.text((x, y), text, font=font, fill=col)
        x += draw.textlength(text, font=font)
    y += LINE_H

# ── Cursor blink block on the last line (just静态) ──
# (we drew it as a "_" so no extra block needed)

OUT = "/var/minis/workspace/angles-cli/docs/assets/angles-demo.png"
img.save(OUT, "PNG")
print(f"✅ saved: {OUT}  ({W}x{H})")
