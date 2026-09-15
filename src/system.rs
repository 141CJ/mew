#[cfg(target_os = "linux")]
use std::fs;

#[derive(Debug, Clone, Default)]
pub struct SystemTelemetry {
    pub cpu_usage: Option<f32>,
    pub cpu_freq_ghz: Option<f32>,
    pub ram_used_gb: Option<f32>,
    pub ram_total_gb: Option<f32>,
    pub ram_percent: Option<f32>,
    pub disk_used_gb: Option<f32>,
    pub disk_total_gb: Option<f32>,
    pub disk_percent: Option<f32>,
    pub uptime_formatted: Option<String>,
}

// sample /proc/stat twice with a brief sleep to calculate instantaneous cpu utilization without a persistent daemon
#[cfg(target_os = "linux")]
pub fn read_cpu_usage() -> Option<f32> {
    fn read_stat_line() -> Option<(u64, u64)> {
        let content = fs::read_to_string("/proc/stat").ok()?;
        let first_line = content.lines().next()?;
        if !first_line.starts_with("cpu ") {
            return None;
        }
        let parts: Vec<u64> = first_line
            .split_whitespace()
            .skip(1)
            .filter_map(|s| s.parse::<u64>().ok())
            .collect();
        if parts.len() < 4 {
            return None;
        }
        let idle = parts[3] + parts.get(4).copied().unwrap_or(0);
        let total: u64 = parts.iter().sum();
        Some((idle, total))
    }

    let (idle1, total1) = read_stat_line()?;
    std::thread::sleep(std::time::Duration::from_millis(15));
    let (idle2, total2) = read_stat_line()?;

    let total_delta = total2.saturating_sub(total1);
    let idle_delta = idle2.saturating_sub(idle1);

    if total_delta == 0 {
        return None;
    }

    let used_delta = total_delta.saturating_sub(idle_delta);
    let usage = (used_delta as f32 / total_delta as f32) * 100.0;
    Some(usage.clamp(0.0, 100.0))
}

#[cfg(not(target_os = "linux"))]
pub fn read_cpu_usage() -> Option<f32> {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_cpu_usage();
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_cpu_usage();
    let usage = sys.global_cpu_usage();
    if usage.is_finite() {
        Some(usage.clamp(0.0, 100.0))
    } else {
        None
    }
}

// None when the file is absent (vms/containers often lack cpufreq)
#[cfg(target_os = "linux")]
pub fn read_cpu_freq() -> Option<f32> {
    let path = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq";
    let content = fs::read_to_string(path).ok()?;
    let khz: f32 = content.trim().parse().ok()?;
    Some(khz / 1_000_000.0)
}

#[cfg(not(target_os = "linux"))]
pub fn read_cpu_freq() -> Option<f32> {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_cpu_frequency();
    let cpus = sys.cpus();
    if cpus.is_empty() {
        return None;
    }
    let sum_mhz: u64 = cpus.iter().map(|c| c.frequency()).sum();
    let avg_mhz = sum_mhz as f32 / cpus.len() as f32;
    if avg_mhz <= 0.0 || !avg_mhz.is_finite() {
        return None;
    }
    Some(avg_mhz / 1000.0)
}

#[cfg(target_os = "linux")]
pub fn read_ram() -> Option<(f32, f32, f32)> {
    let content = fs::read_to_string("/proc/meminfo").ok()?;
    let mut total_kb: Option<u64> = None;
    let mut avail_kb: Option<u64> = None;

    for line in content.lines() {
        if line.starts_with("MemTotal:") {
            total_kb = line
                .split_whitespace()
                .nth(1)
                .and_then(|v| v.parse::<u64>().ok());
        } else if line.starts_with("MemAvailable:") {
            avail_kb = line
                .split_whitespace()
                .nth(1)
                .and_then(|v| v.parse::<u64>().ok());
        }
        if total_kb.is_some() && avail_kb.is_some() {
            break;
        }
    }

    match (total_kb, avail_kb) {
        (Some(total), Some(avail)) if total > 0 => {
            let used = total.saturating_sub(avail);
            let total_gb = total as f32 / (1024.0 * 1024.0);
            let used_gb = used as f32 / (1024.0 * 1024.0);
            let percent = (used as f32 / total as f32) * 100.0;
            Some((used_gb, total_gb, percent))
        }
        _ => None,
    }
}

#[cfg(not(target_os = "linux"))]
pub fn read_ram() -> Option<(f32, f32, f32)> {
    let mut sys = sysinfo::System::new();
    sys.refresh_memory();
    let total = sys.total_memory();
    if total == 0 {
        return None;
    }
    let available = sys.available_memory();
    let used = total.saturating_sub(available);
    let total_gb = total as f32 / (1024.0 * 1024.0 * 1024.0);
    let used_gb = used as f32 / (1024.0 * 1024.0 * 1024.0);
    let percent = (used as f32 / total as f32) * 100.0;
    Some((used_gb, total_gb, percent))
}

// read disk metrics via libc statvfs (correct struct layout on every
// platform, unlike a hand-rolled extern block)
#[cfg(target_os = "linux")]
pub fn read_disk() -> Option<(f32, f32, f32)> {
    use std::ffi::CString;
    use std::mem::MaybeUninit;

    let c_path = CString::new("/").ok()?;
    let mut stat = MaybeUninit::<libc::statvfs>::uninit();

    // statvfs returns 0 on success; buf is initialized on that path only
    if unsafe { libc::statvfs(c_path.as_ptr(), stat.as_mut_ptr()) } != 0 {
        return None;
    }
    let stat = unsafe { stat.assume_init() };

    let total_bytes = stat.f_blocks * stat.f_frsize;
    let free_bytes = stat.f_bfree * stat.f_frsize;

    if total_bytes == 0 {
        return None;
    }

    let used_bytes = total_bytes.saturating_sub(free_bytes);
    let total_gb = total_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
    let used_gb = used_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
    let percent = (used_bytes as f32 / total_bytes as f32) * 100.0;

    Some((used_gb, total_gb, percent))
}

#[cfg(not(target_os = "linux"))]
pub fn read_disk() -> Option<(f32, f32, f32)> {
    use sysinfo::Disks;
    let cwd = std::env::current_dir().ok();
    let disks = Disks::new_with_refreshed_list();
    let mut list: Vec<_> = disks.iter().collect();
    if list.is_empty() {
        return None;
    }
    if let Some(dir) = cwd.as_deref() {
        let mut best: Option<&sysinfo::Disk> = None;
        let mut best_len = 0usize;
        for disk in &list {
            let mount = disk.mount_point();
            if dir.starts_with(mount) {
                let len = mount.as_os_str().len();
                if len >= best_len {
                    best_len = len;
                    best = Some(*disk);
                }
            }
        }
        if let Some(disk) = best {
            let total_bytes = disk.total_space();
            let available = disk.available_space();
            if total_bytes == 0 {
                return None;
            }
            let used_bytes = total_bytes.saturating_sub(available);
            let total_gb = total_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
            let used_gb = used_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
            let percent = (used_bytes as f32 / total_bytes as f32) * 100.0;
            return Some((used_gb, total_gb, percent));
        }
    }
    list.sort_by_key(|d| std::cmp::Reverse(d.total_space()));
    let disk = list.into_iter().next()?;
    let total_bytes = disk.total_space();
    if total_bytes == 0 {
        return None;
    }
    let available = disk.available_space();
    let used_bytes = total_bytes.saturating_sub(available);
    let total_gb = total_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
    let used_gb = used_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
    let percent = (used_bytes as f32 / total_bytes as f32) * 100.0;
    Some((used_gb, total_gb, percent))
}

#[cfg(target_os = "linux")]
pub fn read_uptime() -> Option<String> {
    let content = fs::read_to_string("/proc/uptime").ok()?;
    let first = content.split_whitespace().next()?;
    let seconds_float: f64 = first.parse().ok()?;
    let total_secs = seconds_float as u64;

    let days = total_secs / 86400;
    let rem = total_secs % 86400;
    let hours = rem / 3600;
    let rem = rem % 3600;
    let mins = rem / 60;

    if days > 0 {
        Some(format!("{}d {}h {}m", days, hours, mins))
    } else if hours > 0 {
        Some(format!("{}h {}m", hours, mins))
    } else {
        Some(format!("{}m", mins))
    }
}

#[cfg(not(target_os = "linux"))]
pub fn read_uptime() -> Option<String> {
    let total_secs = sysinfo::System::uptime();
    if total_secs == 0 {
        return Some("0m".to_string());
    }
    let days = total_secs / 86400;
    let rem = total_secs % 86400;
    let hours = rem / 3600;
    let rem = rem % 3600;
    let mins = rem / 60;
    if days > 0 {
        Some(format!("{}d {}h {}m", days, hours, mins))
    } else if hours > 0 {
        Some(format!("{}h {}m", hours, mins))
    } else {
        Some(format!("{}m", mins))
    }
}

// os name (from /etc/os-release PRETTY_NAME or NAME, falling back to std::env::consts::OS)
#[cfg(target_os = "linux")]
pub fn read_os_name() -> String {
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("PRETTY_NAME=") {
                let val = line.trim_start_matches("PRETTY_NAME=").trim_matches('"');
                return val.to_lowercase();
            }
        }
        for line in content.lines() {
            if line.starts_with("NAME=") {
                let val = line.trim_start_matches("NAME=").trim_matches('"');
                return val.to_lowercase();
            }
        }
    }

    if let Some(long) = sysinfo::System::long_os_version()
        && !long.trim().is_empty()
    {
        return long.to_lowercase();
    }

    std::env::consts::OS.to_lowercase()
}

#[cfg(not(target_os = "linux"))]
pub fn read_os_name() -> String {
    if let Some(long) = sysinfo::System::long_os_version()
        && !long.trim().is_empty()
    {
        return long.to_lowercase();
    }
    if let Some(name) = sysinfo::System::name() {
        if let Some(ver) = sysinfo::System::os_version()
            && !ver.trim().is_empty()
        {
            return format!("{} {}", name, ver).to_lowercase();
        }
        if !name.trim().is_empty() {
            return name.to_lowercase();
        }
    }
    std::env::consts::OS.to_lowercase()
}

// distro-specific nerd font icon matching fastfetch / starship
pub fn os_icon() -> &'static str {
    let os = read_os_name();
    if os.contains("cachy") || os.contains("arch") {
        "\u{f303}" // nf-linux-archlinux
    } else if os.contains("ubuntu") {
        "\u{f31b}" // nf-linux-ubuntu
    } else if os.contains("fedora") {
        "\u{f30a}" // nf-linux-fedora
    } else if os.contains("debian") {
        "\u{f306}" // nf-linux-debian
    } else if os.contains("nix") {
        "\u{f313}" // nf-linux-nixos
    } else if os.contains("darwin") || os.contains("mac") {
        "\u{f179}" // nf-fa-apple
    } else if os.contains("windows") {
        "\u{f17a}" // nf-fa-windows
    } else {
        "\u{e712}" // nf-dev-linux
    }
}

// live session info for the context grid: login shell + terminal emulator.
// both are real reads (never fabricated); lowercased for the house style.
#[cfg(not(windows))]
pub fn read_shell_info() -> (String, String) {
    let shell = std::env::var("SHELL")
        .ok()
        .and_then(|s| s.rsplit('/').next().map(|b| b.to_string()))
        .unwrap_or_else(|| "—".to_string())
        .to_lowercase();
    let term = std::env::var("TERM_PROGRAM")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| std::env::var("TERMINAL").ok())
        .unwrap_or_else(|| std::env::var("TERM").unwrap_or_else(|_| "—".to_string()))
        .to_lowercase();
    (shell, term)
}

#[cfg(windows)]
pub fn read_shell_info() -> (String, String) {
    let shell = std::env::var("SHELL")
        .ok()
        .and_then(|s| {
            s.rsplit(['/', '\\'])
                .next()
                .map(|b| b.trim_end_matches(".exe").to_string())
        })
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            std::env::var("ComSpec").ok().and_then(|s| {
                s.rsplit('\\')
                    .next()
                    .map(|b| b.trim_end_matches(".exe").to_string())
            })
        })
        .unwrap_or_else(|| "—".to_string())
        .to_lowercase();
    let term = std::env::var("TERM_PROGRAM")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            std::env::var("WT_SESSION")
                .ok()
                .filter(|s| !s.trim().is_empty())
                .map(|_| "windows terminal".to_string())
        })
        .or_else(|| std::env::var("TERMINAL").ok())
        .unwrap_or_else(|| std::env::var("TERM").unwrap_or_else(|_| "—".to_string()))
        .to_lowercase();
    (shell, term)
}

// rustc version discovery (one subprocess per process at most: the binary
// renders once and exits, so there is nothing worth memoizing here)
pub fn read_rustc_version() -> Option<String> {
    let output = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    // e.g. "rustc 1.85.0 (4d91de4e4 2025-02-17)" -> "rustc 1.85.0"
    // (suffixes like "-nightly" are stripped for a stable width)
    let words: Vec<&str> = stdout.split_whitespace().collect();
    if words.len() >= 2 {
        let ver: String = words[1]
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        if !ver.is_empty() {
            return Some(format!("rustc {}", ver.to_lowercase()));
        }
    }
    None
}

// language-aware toolchain probe for the env row: runs exactly one
// subprocess for the detected project language (same budget as the old
// always-rustc call). kotlin is deliberately excluded: kotlinc needs a
// jvm boot measured in seconds, far outside the latency budget.
pub fn read_toolchain_version(lang: &str) -> Option<String> {
    const VERSION_FLAG: &[&str] = &["--version"];
    const VERSION_CMD: &[&str] = &["version"];
    const V_FLAG: &[&str] = &["-v"];

    let key = lang.to_lowercase();
    if key == "kotlin" {
        return None;
    }
    if key == "shell" {
        return shell_toolchain();
    }
    let (name, cmd, args): (Option<&str>, &str, &[&str]) = match key.as_str() {
        "rust" => (Some("rustc"), "rustc", VERSION_FLAG),
        "python" => (Some("python"), "python3", VERSION_FLAG),
        "go" => (Some("go"), "go", VERSION_CMD),
        "javascript" | "typescript" | "react" => (Some("node"), "node", VERSION_FLAG),
        "c" => (None, "cc", VERSION_FLAG),
        "c++" => (None, "c++", VERSION_FLAG),
        "java" => (None, "java", VERSION_FLAG),
        "swift" => (Some("swift"), "swift", VERSION_FLAG),
        "ruby" => (Some("ruby"), "ruby", VERSION_FLAG),
        "php" => (Some("php"), "php", VERSION_FLAG),
        "zig" => (Some("zig"), "zig", VERSION_CMD),
        "lua" => (Some("lua"), "lua", V_FLAG),
        "haskell" => (Some("ghc"), "ghc", VERSION_FLAG),
        _ => return None,
    };

    let line = probe_version(cmd, args).or_else(|| {
        if key == "python" {
            probe_version("python", args)
        } else if key == "c" {
            probe_version("gcc", args)
                .or_else(|| probe_version("clang", args))
                .or_else(|| probe_version("cl", args))
        } else if key == "c++" {
            probe_version("g++", args)
                .or_else(|| probe_version("clang++", args))
                .or_else(|| probe_version("cl", args))
        } else {
            None
        }
    })?;
    let ver = first_version_token(&line)?;
    // c, c++ and java compilers vary by vendor (gcc/clang, openjdk/temurin),
    // so the display name comes from the output, not the mapping above.
    let display = match name {
        Some(n) => n,
        None => {
            let low = line.to_lowercase();
            match key.as_str() {
                "c" => {
                    if low.contains("clang") {
                        "clang"
                    } else {
                        "cc"
                    }
                }
                "c++" => {
                    if low.contains("clang") {
                        "clang++"
                    } else {
                        "c++"
                    }
                }
                _ => {
                    if low.contains("openjdk") {
                        "openjdk"
                    } else {
                        "java"
                    }
                }
            }
        }
    };
    Some(format!("{} {}", display, ver))
}

// run `<cmd> <args>` and return the first output line, trying stdout then
// stderr (some tools, e.g. java, report on stderr). none on any failure.
fn probe_version(cmd: &str, args: &[&str]) -> Option<String> {
    let output = std::process::Command::new(cmd).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = if output.stdout.is_empty() {
        String::from_utf8_lossy(&output.stderr).into_owned()
    } else {
        String::from_utf8_lossy(&output.stdout).into_owned()
    };
    let line = text.lines().next()?.trim();
    if line.is_empty() {
        return None;
    }
    Some(line.to_string())
}

// first whitespace-separated token that looks like a version: leading
// non-digits stripped ("v", "go"), at least one dot ("5.2.21(1)-release"
// yields "5.2.21"). skips a literal "version" token ("go version ...").
fn first_version_token(line: &str) -> Option<String> {
    for tok in line.split_whitespace() {
        if tok.eq_ignore_ascii_case("version") {
            continue;
        }
        let stripped: String = tok
            .trim_start_matches(|c: char| !c.is_ascii_digit())
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        if stripped.contains('.') && stripped.starts_with(|c: char| c.is_ascii_digit()) {
            return Some(stripped);
        }
    }
    None
}

// toolchain of the login shell itself ($SHELL --version), for shell repos.
fn shell_toolchain() -> Option<String> {
    if cfg!(windows) {
        return windows_shell_toolchain();
    }
    let shell_path = std::env::var("SHELL").ok()?;
    let base = shell_path.rsplit('/').next().filter(|s| !s.is_empty())?;
    let line = probe_version(base, &["--version"])?;
    let ver = first_version_token(&line)?;
    Some(format!("{} {}", base.to_lowercase(), ver))
}

#[cfg(windows)]
fn windows_shell_toolchain() -> Option<String> {
    let shell_path = std::env::var("SHELL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| std::env::var("ComSpec").ok())?;
    let base_raw = shell_path
        .rsplit(['/', '\\'])
        .next()
        .filter(|s| !s.is_empty())?;
    let base = base_raw.trim_end_matches(".exe").trim_end_matches(".EXE");
    if base.is_empty() {
        return None;
    }
    let low = base.to_lowercase();
    if low == "powershell" || low == "powershell_ise" {
        let output = std::process::Command::new(base_raw)
            .args([
                "-NoProfile",
                "-Command",
                "$PSVersionTable.PSVersion.ToString()",
            ])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let text = String::from_utf8_lossy(&output.stdout);
        let line = text.lines().next()?.trim().to_string();
        let ver = first_version_token(&line)?;
        return Some(format!("{} {}", low, ver));
    }
    if low == "cmd" {
        let output = std::process::Command::new(base_raw)
            .args(["/c", "ver"])
            .output()
            .ok()?;
        let text = if output.stdout.is_empty() {
            String::from_utf8_lossy(&output.stderr).into_owned()
        } else {
            String::from_utf8_lossy(&output.stdout).into_owned()
        };
        let line = text.lines().next()?.trim().to_string();
        let ver = first_version_token(&line)?;
        return Some(format!("{} {}", low, ver));
    }
    let line =
        probe_version(base, &["--version"]).or_else(|| probe_version(base_raw, &["--version"]))?;
    let ver = first_version_token(&line)?;
    Some(format!("{} {}", low, ver))
}

#[cfg(not(windows))]
fn windows_shell_toolchain() -> Option<String> {
    None
}

// collect full system telemetry for --full mode
pub fn collect_system_telemetry() -> SystemTelemetry {
    let (ram_used_gb, ram_total_gb, ram_percent) = match read_ram() {
        Some((u, t, p)) => (Some(u), Some(t), Some(p)),
        None => (None, None, None),
    };

    let (disk_used_gb, disk_total_gb, disk_percent) = match read_disk() {
        Some((u, t, p)) => (Some(u), Some(t), Some(p)),
        None => (None, None, None),
    };

    SystemTelemetry {
        cpu_usage: read_cpu_usage(),
        cpu_freq_ghz: read_cpu_freq(),
        ram_used_gb,
        ram_total_gb,
        ram_percent,
        disk_used_gb,
        disk_total_gb,
        disk_percent,
        uptime_formatted: read_uptime(),
    }
}
