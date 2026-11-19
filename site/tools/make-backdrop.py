#!/usr/bin/env python3
"""Generates site/assets/backdrop.svg: a dim, static "stack of editor panes" backdrop
(file tabs, line numbers, syntax-coloured token bars). Deterministic, so re-running it
produces the same file."""
import random
from pathlib import Path

W, H = 1600, 1000
rng = random.Random(1987)
COLORS = ["#ff9a3c", "#9b6bff", "#2ee6f6", "#ff4fa3", "#b6ff3b", "#c9d1dc"]
WEIGHTS = [3, 3, 2, 1, 1, 6]
out = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" preserveAspectRatio="xMidYMin slice">',
       '<rect width="100%" height="100%" fill="#07060c"/>']


def pane(x, y, w, h, title, depth):
    o = 0.55 - depth * 0.15
    out.append(f'<g opacity="{o:.2f}">')
    out.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="8" fill="#0e0c17" stroke="#26213a"/>')
    out.append(f'<rect x="{x}" y="{y}" width="{w}" height="28" rx="8" fill="#141120"/>')
    for i, c in enumerate(["#ff4fa3", "#ff9a3c", "#b6ff3b"]):
        out.append(f'<circle cx="{x + 16 + i * 14}" cy="{y + 14}" r="4" fill="{c}" opacity=".7"/>')
    out.append(f'<rect x="{x + 70}" y="{y + 9}" width="{len(title) * 7}" height="10" rx="3" fill="#3a3352"/>')
    line_y, n = y + 46, 1
    while line_y < y + h - 14:
        out.append(f'<rect x="{x + 12}" y="{line_y}" width="{8 if n < 10 else 14}" height="6" rx="2" fill="#2b2640"/>')
        cx, indent = x + 40 + rng.choice([0, 0, 16, 16, 32, 48]), 0
        for _ in range(rng.randint(0, 6)):
            tw = rng.randint(14, 90)
            if cx + tw > x + w - 16:
                break
            color = rng.choices(COLORS, WEIGHTS)[0]
            out.append(f'<rect x="{cx}" y="{line_y}" width="{tw}" height="6" rx="2" fill="{color}" opacity=".55"/>')
            cx += tw + 8
        line_y += 15
        n += 1
    out.append("</g>")


pane(60, 120, 520, 420, "src/router.rs", 0)
pane(640, 60, 560, 520, "src/api/items.rs", 1)
pane(1250, 160, 400, 360, "Cargo.toml", 2)
pane(-40, 600, 600, 380, "proto/greeter.proto", 1)
pane(620, 640, 520, 340, "templates/index.html", 0)
pane(1180, 580, 480, 400, "src/realtime/mod.rs", 2)
out.append("</svg>")
target = Path(__file__).resolve().parents[1] / "assets" / "backdrop.svg"
target.write_text("\n".join(out) + "\n")
print(f"wrote {target} ({target.stat().st_size // 1024} KB)")
