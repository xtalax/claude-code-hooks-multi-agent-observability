#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "python-dotenv",
# ]
# ///

"""
Status Line v6 - Runic Powerline
Mirrors the Runic fish theme segment pattern exactly:
  ─ content  ─  content  ...
Each segment is individually enclosed with left/right arrows,
connected by purple ─ dashes.

Reads colors from ~/.config/omarchy/current/theme/colors.toml.
"""

import json
import os
import re
import subprocess
import sys

try:
    from dotenv import load_dotenv

    load_dotenv()
except ImportError:
    pass

# Powerline / Runic glyphs
SEP_L = "\uE0B2"  #  (left-pointing, enters segment)
SEP_R = "\uE0B0"  #  (right-pointing, exits segment)
BRANCH = "\uE0A0"  #
RUNIC = "ᛝ"
CONN = "─"

RESET = "\033[0m"

FALLBACK_COLORS = {
    "accent": "#7daea3",
    "foreground": "#d4be98",
    "background": "#282828",
    "color0": "#3c3836",
    "color1": "#ea6962",
    "color2": "#a9b665",
    "color3": "#d8a657",
    "color4": "#7daea3",
    "color5": "#d3869b",
    "color6": "#89b482",
    "color7": "#d4be98",
}

SEMANTIC_MAP = {
    "purple": "color5",
    "yellow": "color3",
    "orange": "color1",
    "turquoise": "color6",
    "blue": "color4",
    "green": "color2",
    "red": "color1",
    "black": "background",
    "white": "foreground",
    "accent": "accent",
    "bg1": "color0",
}


def load_theme_colors():
    theme_path = os.path.expanduser(
        "~/.config/omarchy/current/theme/colors.toml"
    )
    raw = {}
    try:
        with open(theme_path) as f:
            for line in f:
                line = line.strip()
                if "=" in line and not line.startswith("#"):
                    key, val = line.split("=", 1)
                    raw[key.strip()] = val.strip().strip('"')
    except OSError:
        raw = FALLBACK_COLORS

    colors = {}
    for name, toml_key in SEMANTIC_MAP.items():
        hex_color = raw.get(toml_key) or FALLBACK_COLORS.get(toml_key, "#888888")
        colors[name] = hex_to_rgb(hex_color)
    return colors


def hex_to_rgb(h):
    h = h.lstrip("#")
    return (int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16))


COLORS = load_theme_colors()


def fg(name):
    r, g, b = COLORS[name]
    return f"\033[38;2;{r};{g};{b}m"


def bg_c(name):
    r, g, b = COLORS[name]
    return f"\033[48;2;{r};{g};{b}m"


def runic_segment(bg_name, fg_name, text):
    """Draw a single runic segment: content"""
    return (
        f"{RESET}{fg(bg_name)}{SEP_L}"       # ← enter (left arrow, fg=segment color)
        f"{bg_c(bg_name)}{fg(fg_name)}{text}" # content
        f"{RESET}{fg(bg_name)}{SEP_R}"        # → exit (right arrow, fg=segment color)
    )


def connector():
    """Purple ─ between segments, matching runic fish prompt."""
    return f"{fg('purple')}{CONN}"


def usage_color(pct):
    if pct < 50:
        return "green"
    elif pct < 75:
        return "yellow"
    elif pct < 90:
        return "orange"
    return "red"


def progress_bar(pct, width=10):
    filled = int((pct / 100) * width)
    empty = width - filled
    return f"{fg(usage_color(pct))}{'█' * filled}{fg('bg1')}{'░' * empty}"


MODEL_MAX_CONTEXT = {
    "claude-opus-4-6-1M": 1_000_000,
    "claude-sonnet-4-6-1M": 1_000_000,
    # Opus 4.6 and Sonnet 4.6 now default to 1M context windows
    "claude-opus-4-6": 1_000_000,
    "claude-sonnet-4-6": 1_000_000,
    "claude-haiku-4-5": 200_000,
    "claude-sonnet-4-5": 200_000,
}
DEFAULT_CONTEXT = 200_000


def get_max_context(data):
    """Derive max context from context_window data, falling back to model ID lookup."""
    ctx = data.get("context_window", {})
    size = ctx.get("context_window_size")
    if size and size > 0:
        return size
    model_id = data.get("model", {}).get("id", "")
    return MODEL_MAX_CONTEXT.get(model_id, DEFAULT_CONTEXT)


def fmt_tokens(tokens):
    if tokens is None:
        return "0"
    if tokens < 1000:
        return str(int(tokens))
    elif tokens < 1000000:
        return f"{tokens / 1000:.1f}k"
    return f"{tokens / 1000000:.2f}M"


def get_git_info(cwd):
    if not cwd:
        return None

    def run(args):
        try:
            r = subprocess.run(
                ["git"] + args, cwd=cwd,
                capture_output=True, text=True, timeout=3,
            )
            return r.stdout.strip() if r.returncode == 0 else None
        except Exception:
            return None

    branch = run(["rev-parse", "--abbrev-ref", "HEAD"])
    if not branch:
        return None

    diff_stat = run(["diff", "--shortstat", "HEAD"])
    added = removed = 0
    if diff_stat:
        m = re.search(r"(\d+) insertion", diff_stat)
        if m:
            added = int(m.group(1))
        m = re.search(r"(\d+) deletion", diff_stat)
        if m:
            removed = int(m.group(1))

    dirty = bool(run(["status", "--porcelain"]))

    return {"branch": branch, "added": added, "removed": removed, "dirty": dirty}


def generate_status_line(data):
    model = data.get("model", {}).get("display_name", "Claude")
    sid = (data.get("session_id", "") or "--------")[:8]

    win = get_max_context(data)
    ctx = data.get("context_window", {})
    pct = ctx.get("used_percentage", 0) or 0
    remaining = int(win * ((100 - pct) / 100))

    git = get_git_info(data.get("cwd", ""))

    segments = []

    # Segment 1: ᛝ model (purple bg, like waybar #custom-omarchy)
    segments.append(runic_segment("purple", "black", f" {RUNIC} {model} "))

    # Segment 2: git branch + diff stats (yellow=dirty, green=clean)
    if git:
        br = git["branch"]
        if len(br) > 20:
            br = br[:18] + ".."

        git_bg = "yellow" if git["dirty"] else "green"

        # Build git text with colored +/- inline
        git_text = f" {BRANCH} {br}"
        if git["added"]:
            git_text += f" {fg('green')}+{git['added']}"
        if git["removed"]:
            git_text += f" {fg('red')}-{git['removed']}"
        # Restore segment fg for ± indicator
        if git["dirty"]:
            git_text += f" {fg('black')}±"
        git_text += " "

        segments.append(runic_segment(git_bg, "black", git_text))

    # Segment 3: context usage bar + percentage + window size (turquoise, like waybar #clock)
    u_bg = "turquoise"
    bar = progress_bar(pct)
    segments.append(runic_segment(u_bg, "black", f" {bar} {fg('black')}{bg_c(u_bg)}{pct:.0f}% of {fmt_tokens(win)} "))

    # Segment 4: tokens remaining (blue, like waybar accent/workspaces)
    segments.append(runic_segment("blue", "black", f" ~{fmt_tokens(remaining)} left "))

    # Segment 5: session ID (dim bg1)
    segments.append(runic_segment("bg1", "white", f" {sid} "))

    return connector().join(segments) + RESET


def main():
    try:
        data = json.loads(sys.stdin.read())
        print(generate_status_line(data))
    except json.JSONDecodeError:
        print(f"{fg('red')}[Claude] {RUNIC} Error: Invalid JSON{RESET}")
    except Exception as e:
        print(f"{fg('red')}[Claude] {RUNIC} Error: {e}{RESET}")
    sys.exit(0)


if __name__ == "__main__":
    main()
