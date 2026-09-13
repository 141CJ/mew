#!/usr/bin/env bash
set -euo pipefail

# check for rust toolchain
if ! command -v cargo >/dev/null 2>&1; then
    echo "mew: cargo not found. installing rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

echo "mew: compiling release binary..."
cargo build --release --locked

echo "mew: stripping binary..."
strip target/release/mew

echo "mew: installing binary to /usr/local/bin..."
if [ -w /usr/local/bin ]; then
    install -m 755 target/release/mew /usr/local/bin/mew
else
    sudo install -m 755 target/release/mew /usr/local/bin/mew
fi

# provision default configuration file if absent
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/mew"
THEME_FILE="$CONFIG_DIR/theme.toml"

if [ ! -f "$THEME_FILE" ]; then
    echo "mew: provisioning default theme at $THEME_FILE..."
    mkdir -p "$CONFIG_DIR"
    cat << 'EOF' > "$THEME_FILE"
[colors]
border       = "#45475a"
primary      = "#cdd6f4"
muted        = "#6c7086"
mascot       = "#f2cdcd"
task_active  = "#b4befe"
gauge_fill   = "#89b4fa"
gauge_empty  = "#313244"
status_clean = "#a6e3a1"
status_dirty = "#f9e2af"
status_error = "#f38ba8"
# per-item accents: row dots, card titles, clock gradient stops
dot_workspace = "#89dceb"
dot_git       = "#fab387"
dot_env       = "#cba6f7"
dot_task      = "#a6e3a1"
dot_mood      = "#f5c2e7"
title_mew       = "#cba6f7"
title_sys       = "#a6e3a1"
title_context   = "#fab387"
title_telem     = "#f5c2e7"
title_cmd       = "#94e2d5"
# signature fastfetch palette strip
dots = ["#f38ba8", "#fab387", "#f9e2af", "#a6e3a1", "#94e2d5", "#89b4fa", "#cba6f7", "#f5c2e7"]
EOF
fi

# optional hyprland integration:
# only prompt/append if ~/.config/hypr/hyprland.conf already exists.
# we never create or edit a user's window manager configuration unconditionally.
HYPR_CONF="${XDG_CONFIG_HOME:-$HOME/.config}/hypr/hyprland.conf"
if [ -f "$HYPR_CONF" ]; then
    if ! grep -q "bind = \$mainMod, M, exec" "$HYPR_CONF" 2>/dev/null; then
        echo "mew: hyprland config detected."
        echo "mew: to bind super+m to launch mew in a floating terminal, add:"
        echo "     bind = \$mainMod, M, exec, alacritty --class mew -e mew"
    fi
fi

echo "mew: installation complete. run 'mew hook <shell>' to integrate with your prompt."
