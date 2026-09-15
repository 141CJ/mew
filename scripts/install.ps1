$ErrorActionPreference = "Stop"
$InstallDir = Join-Path $HOME ".local\bin"
$ThemeDir = Join-Path $env:APPDATA "mew"
$ThemeFile = Join-Path $ThemeDir "theme.toml"

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "mew: cargo not found. installing rustup..."
    $RustupInit = Join-Path $env:TEMP "rustup-init.exe"
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $RustupInit -UseBasicParsing
    & $RustupInit -y --default-toolchain stable
    $CargoBin = Join-Path $HOME ".cargo\bin"
    $env:Path = "$CargoBin;$env:Path"
}

Write-Host "mew: compiling release binary..."
cargo build --release --locked

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Write-Host "mew: installing binary to $InstallDir..."
Copy-Item "target\release\mew.exe" (Join-Path $InstallDir "mew.exe") -Force

$PathParts = $env:Path -split ";"
if ($PathParts -notcontains $InstallDir) {
    Write-Host "mew: adding $InstallDir to user PATH..."
    $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($UserPath -notlike "*$InstallDir*") {
        [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    }
    $env:Path = "$env:Path;$InstallDir"
}

if (-not (Test-Path $ThemeFile)) {
    Write-Host "mew: provisioning default theme at $ThemeFile..."
    New-Item -ItemType Directory -Force -Path $ThemeDir | Out-Null
    @'
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
title_mew       = "#cba6f7"
title_sys       = "#a6e3a1"
title_context   = "#fab387"
title_telem     = "#f5c2e7"
title_cmd       = "#94e2d5"
dots = ["#f38ba8", "#fab387", "#f9e2af", "#a6e3a1", "#94e2d5", "#89b4fa", "#cba6f7", "#f5c2e7"]
'@ | Out-File -FilePath $ThemeFile -Encoding utf8
}

Write-Host "mew: installation complete. run 'mew hook powershell' to integrate with your prompt."
