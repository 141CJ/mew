use clap::ValueEnum;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    Nu,
}

pub const BASH_HOOK: &str = r#"__auto_mew_on_cd() {
    if [ "$PWD" != "$__LAST_PWD" ]; then
        __LAST_PWD="$PWD"
        if command -v mew >/dev/null 2>&1 && git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
            mew
        fi
    fi
}
PROMPT_COMMAND="__auto_mew_on_cd;$PROMPT_COMMAND""#;

pub const ZSH_HOOK: &str = r#"chpwd() {
    if command -v mew >/dev/null 2>&1 && git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        mew
    fi
}"#;

pub const FISH_HOOK: &str = r#"function --on-variable PWD __auto_mew
    if type -q mew; and status is-interactive; and git rev-parse --is-inside-work-tree >/dev/null 2>&1
        mew
    end
end"#;

pub const NU_HOOK: &str = r#"$env.config = ($env.config | default {} hooks)
$env.config.hooks = ($env.config.hooks | default {} env_change)
$env.config.hooks.env_change = ($env.config.hooks.env_change | default [] PWD)

$env.config.hooks.env_change.PWD = (
    $env.config.hooks.env_change.PWD | append {||
        if (which mew | is-not-empty) and (do { git rev-parse --is-inside-work-tree } | complete | get exit_code) == 0 {
            mew
        }
    }
)"#;

pub fn print_hook(shell: Shell) {
    match shell {
        Shell::Bash => println!("{}", BASH_HOOK),
        Shell::Zsh => println!("{}", ZSH_HOOK),
        Shell::Fish => println!("{}", FISH_HOOK),
        Shell::Nu => println!("{}", NU_HOOK),
    }
}
