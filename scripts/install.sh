#!/usr/bin/env bash
set -euo pipefail

REPO_URL="https://github.com/programmersd21/mew.git"
INSTALL_DIR="${MEW_INSTALL_DIR:-$HOME/.local/share/mew}"
BIN_DIR="${MEW_BIN_DIR:-$HOME/.local/bin}"

log() {
    printf 'mew: %s\n' "$1"
}

if ! command -v cargo >/dev/null 2>&1; then
    log "cargo not found. installing rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    . "$HOME/.cargo/env"
fi

if [ -f "Cargo.toml" ] && [ -f "src/main.rs" ]; then
    MEW_SOURCE_DIR="$(pwd)"
else
    printf '%s\n' \
        "How would you like to install mew?" \
        "" \
        "1) cargo install mew-cli" \
        "2) build from GitHub source" \
        ""

    read -r -p "Choose [1/2]: " choice

    case "$choice" in
        1)
            log "installing mew-cli from crates.io..."
            cargo install mew-cli
            exit 0
            ;;
        2)
            if ! command -v git >/dev/null 2>&1; then
                log "git not found. install git and run this installer again."
                exit 1
            fi

            log "cloning mew..."
            rm -rf "$INSTALL_DIR"
            git clone --depth 1 "$REPO_URL" "$INSTALL_DIR"
            MEW_SOURCE_DIR="$INSTALL_DIR"
            ;;
        *)
            log "invalid choice."
            exit 1
            ;;
    esac
fi

cd "$MEW_SOURCE_DIR"

log "compiling release binary..."
cargo build --release --locked

if command -v strip >/dev/null 2>&1; then
    log "stripping binary..."
    strip target/release/mew
fi

mkdir -p "$BIN_DIR"

log "installing binary..."
install -m 755 target/release/mew "$BIN_DIR/mew"

CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/mew"
THEME_FILE="$CONFIG_DIR/theme.toml"

if [ ! -f "$THEME_FILE" ]; then
    log "provisioning default theme..."
    mkdir -p "$CONFIG_DIR"

    cat <<'EOF' > "$THEME_FILE"
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

dot_workspace = "#89dceb"
dot_git       = "#fab387"
dot_env       = "#cba6f7"
dot_task      = "#a6e3a1"
dot_mood      = "#f5c2e7"

title_mew     = "#cba6f7"
title_sys     = "#a6e3a1"
title_context = "#fab387"
title_telem   = "#f5c2e7"
title_cmd     = "#94e2d5"

dots = [
    "#f38ba8",
    "#fab387",
    "#f9e2af",
    "#a6e3a1",
    "#94e2d5",
    "#89b4fa",
    "#cba6f7",
    "#f5c2e7"
]
EOF
fi

HYPR_CONF="${XDG_CONFIG_HOME:-$HOME/.config}/hypr/hyprland.conf"

if [ -f "$HYPR_CONF" ] && ! grep -q 'bind = \$mainMod, M, exec' "$HYPR_CONF" 2>/dev/null; then
    log "hyprland config detected."
    printf '%s\n' '     bind = $mainMod, M, exec, alacritty --class mew -e mew'
fi

log "installation complete."
log "run 'mew hook <shell>' to integrate with your prompt."
