<div align="center">

# mew

<img src="assets/demo.png" alt="mew demo" width="700">

**a fast terminal card for your project, git state, and machine.**

[![build](https://img.shields.io/github/actions/workflow/status/programmersd21/mew/ci.yml?style=flat-square\&label=build\&labelColor=313244\&color=a6e3a1)](https://github.com/programmersd21/mew/actions)
[![release](https://img.shields.io/github/v/release/programmersd21/mew?style=flat-square\&label=release\&labelColor=313244\&color=cba6f7)](https://github.com/programmersd21/mew/releases)
[![crates.io](https://img.shields.io/crates/v/mew-cli?style=flat-square\&logo=rust\&logoColor=f9e2af\&label=crates.io\&labelColor=313244\&color=f9e2af)](https://crates.io/crates/mew-cli)
[![aur](https://img.shields.io/aur/version/mew-bin?style=flat-square\&logo=archlinux\&logoColor=89dceb\&label=aur\&labelColor=313244\&color=89dceb)](https://aur.archlinux.org/packages/mew-bin)
[![license](https://img.shields.io/github/license/programmersd21/mew?style=flat-square\&label=license\&labelColor=313244\&color=74c7ec)](LICENSE)
[![stars](https://img.shields.io/github/stars/programmersd21/mew?style=flat-square\&label=stars\&labelColor=313244\&color=f9e2af)](https://github.com/programmersd21/mew)

</div>

## what it does

run `mew` in a project and get the useful stuff at a glance:

* git branch, dirty/staged/conflict/ahead state
* project language and live toolchain version
* lines of code and detected coverage
* cpu, frequency, memory, disk, and uptime
* clock, date, timezone, and day progress

inside git worktrees, shell hooks can show it automatically.

outside git, mew becomes a small system readout.

<img src="assets/no_repo.png" alt="mew outside a git repository" width="600">

it runs on demand and reads git state only when invoked, so it stays fast with nothing running in the background.

## mew vs fastfetch

[Fastfetch](https://github.com/fastfetch-cli/fastfetch) is for **your machine**. mew is for **what you're working on**.

|                    | mew | fastfetch |
| ------------------ | --- | --------- |
| git status         | ✓   | —         |
| branch / conflicts | ✓   | —         |
| toolchain          | ✓   | —         |
| lines / coverage   | ✓   | —         |
| system info        | ✓   | ✓         |
| shell git hooks    | ✓   | —         |
| system-focused     | —   | ✓         |
| custom modules     | —   | ✓         |

use fastfetch for a system snapshot. use mew when you want the project in front of you.

## install

```bash
curl -sSf https://raw.githubusercontent.com/programmersd21/mew/main/scripts/install.sh | bash
```

or:

```bash
cargo install mew-cli
```

or, last but not the least, but with **your favourite AUR helper**:

```bash
paru -S mew-bin
```

```bash
yay -S mew-bin
```

## usage

```bash
mew
mew --full
```

## shell hooks

```bash
eval "$(mew hook bash)"
eval "$(mew hook zsh)"
mew hook fish | source
mew hook nu
```

hooks only activate inside git worktrees.

## config

optional theme:

```text
~/.config/mew/theme.toml
```

mew defaults to a Catppuccin-inspired palette and accepts per-element colors.

## requirements

* Linux
* Nerd Font recommended
* Rust 1.85+ if building from source

## license

MIT
