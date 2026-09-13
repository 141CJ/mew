<div align="center">

# mew

<p align="center"><img src="assets/demo.png" alt="mew demo" width="700"/></p>

fast, zero-daemon terminal companion and git workspace dashboard

[![CI](https://img.shields.io/github/actions/workflow/status/programmersd21/mew/ci.yml?style=flat-square&label=build)](https://github.com/programmersd21/mew/actions)
[![Release](https://img.shields.io/github/v/release/programmersd21/mew?style=flat-square&label=release)](https://github.com/programmersd21/mew/releases)
[![Crates.io](https://img.shields.io/crates/v/mew-bin?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/mew-bin)
[![Downloads](https://img.shields.io/crates/d/mew-bin?style=flat-square&label=downloads)](https://crates.io/crates/mew)
[![AUR](https://img.shields.io/aur/version/mew-bin?style=flat-square&logo=archlinux&logoColor=white)](https://aur.archlinux.org/packages/mew-bin)
[![License](https://img.shields.io/github/license/programmersd21/mew?style=flat-square&label=license)](LICENSE)
[![Stars](https://img.shields.io/github/stars/programmersd21/mew?style=flat-square&label=stars)](https://github.com/programmersd21/mew)
[![Issues](https://img.shields.io/github/issues/programmersd21/mew?style=flat-square&label=issues)](https://github.com/programmersd21/mew/issues)

</div>

## features

- sub-10ms render latency with zero background daemons.
- direct libgit2 bindings for clean, non-forking git status introspection.
- reactive 3-line ascii mascot reflecting repository conflict, dirty, staged, and ahead states.
- live gradient clock with date, timezone, and day progress.
- language-aware toolchain readout: probes the active project's compiler (`rustc`, `python`, `node`, `go`…) live, with no placeholders.
- outside git repos the card doubles as a fastfetch-style readout with a live uptime row.
- live linux hardware telemetry (instantaneous `/proc/stat` cpu, scaling frequency, `/proc/meminfo` ram, `statvfs` disk, `/proc/uptime`).
- gitignore-respecting loc counter and automatic coverage report discovery.
- native shell hooks for bash, zsh, fish, and nushell gated strictly to git worktrees.

## installation

install via the installation script:

```bash
curl -sSf https://raw.githubusercontent.com/programmersd21/mew/main/scripts/install.sh | bash
```

or build directly with cargo:

```bash
cargo install --path .
```

or from crates.io:

```bash
cargo install mew-cli
```

## usage

run mew directly in any directory:

```bash
# compact one-shot render (default, fast, shell-friendly)
mew

# expanded view with system health, live clock, shell session, and project telemetry
mew --full
```

## configuration

mew loads user colors from `~/.config/mew/theme.toml` and falls back cleanly to catppuccin mocha if absent:

```toml
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
```

every key is optional: older theme files missing the newer accent keys keep loading with the defaults above.

## fonts

mew renders nerd font icons, so use a patched mono font. first pick is jetbrainsmono nerd font (free, safest default with distinct `1`/`l`/`i`). geist mono nerd font pairs well if you want a quieter voice, berkeley mono if you want character. linux terminal staples: hack, iosevka, cascadia code. size 12-14pt, line-height 1.4-1.6.

## shell integration

add one line to your shell configuration to run mew on cd into git repositories:

**bash** (`~/.bashrc`):
```bash
eval "$(mew hook bash)"
```

**zsh** (`~/.zshrc`):
```zsh
eval "$(mew hook zsh)"
```

**fish** (`~/.config/fish/config.fish`):
```fish
mew hook fish | source
```

**nushell** (`~/.config/nushell/config.nu`):
```nu
mew hook nu
```

## development

run standard checks before submitting pull requests:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

## license

mit
