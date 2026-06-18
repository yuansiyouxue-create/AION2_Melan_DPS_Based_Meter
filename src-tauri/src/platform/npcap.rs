//! Npcap detection and guided installation.
//!
//! The meter relies on the Npcap packet-capture driver. Npcap is a kernel-mode
//! driver, so it cannot run from inside our executable — it must be installed at
//! the OS level. To keep distribution to a single standalone `.exe`, the Npcap
//! installer is embedded into the binary via `include_bytes!` and written to a
//! temp file on demand, so it travels with the exe instead of being an external
//! resource file that a bare exe would leave behind.

#[cfg(windows)]
static NPCAP_INSTALLER: &[u8] = include_bytes!("../../resources/npcap-installer.exe");

/// Returns true if the Npcap (or WinPcap-compatible) runtime is present.
#[cfg(windows)]
pub fn is_installed() -> bool {
    use std::path::Path;
    let sysroot = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
    // Npcap installs wpcap.dll either into System32\Npcap (its own dir) or, in
    // WinPcap-compatible mode, directly into System32 / SysWOW64.
    let candidates = [
        format!("{sysroot}\\System32\\Npcap\\wpcap.dll"),
        format!("{sysroot}\\System32\\wpcap.dll"),
        format!("{sysroot}\\SysWOW64\\Npcap\\wpcap.dll"),
        format!("{sysroot}\\SysWOW64\\wpcap.dll"),
    ];
    candidates.iter().any(|p| Path::new(p).exists())
}

#[cfg(windows)]
fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// If Npcap is missing, ask the user with a native dialog and, on consent,
/// launch the embedded installer (blocking until it exits). Best-effort: any
/// failure is logged and ignored so the app still starts.
#[cfg(windows)]
pub fn ensure_installed() {
    if is_installed() {
        return;
    }

    use windows::core::PCWSTR;
    use windows::Win32::UI::WindowsAndMessaging::*;

    let msg = to_wide(
        "未检测到 Npcap 抓包驱动。\n\n\
         「操你妈612DPS统计」需要 Npcap 才能统计游戏伤害（这是一个系统级驱动，无法内置直接运行，只能安装一次）。\n\n\
         现在安装吗？\n\
         安装向导里请保持默认勾选 “Install Npcap in WinPcap API-compatible Mode”，一路点 Next 即可。",
    );
    let title = to_wide("Melan用 — 需要安装 Npcap 抓包驱动");

    let answer = unsafe {
        MessageBoxW(
            None,
            PCWSTR(msg.as_ptr()),
            PCWSTR(title.as_ptr()),
            MB_YESNO | MB_ICONWARNING | MB_TOPMOST | MB_SETFOREGROUND,
        )
    };
    if answer != IDYES {
        return;
    }

    match run_installer_blocking() {
        Ok(_) => {
            let done = to_wide(
                "Npcap 安装流程已结束。\n\n\
                 请关闭并重新以「管理员身份」运行「Melan用」，伤害统计即可生效。",
            );
            let done_title = to_wide("Melan用 — 安装完成");
            unsafe {
                MessageBoxW(
                    None,
                    PCWSTR(done.as_ptr()),
                    PCWSTR(done_title.as_ptr()),
                    MB_OK | MB_ICONINFORMATION | MB_TOPMOST | MB_SETFOREGROUND,
                );
            }
        }
        Err(e) => tracing::error!("Failed to launch Npcap installer: {e}"),
    }
}

/// Write the embedded installer to a temp file and run it, blocking until the
/// wizard is closed. The app runs elevated, so the child inherits admin rights
/// and the driver install proceeds.
#[cfg(windows)]
fn run_installer_blocking() -> std::io::Result<std::process::ExitStatus> {
    let mut path = std::env::temp_dir();
    path.push("npcap-installer.exe");
    std::fs::write(&path, NPCAP_INSTALLER)?;
    std::process::Command::new(&path).status()
}

/// Launch the bundled Npcap installer on demand (used by the frontend
/// `npcap-missing` prompt). Returns an error string on failure.
#[cfg(windows)]
pub fn launch_installer() -> Result<(), String> {
    run_installer_blocking()
        .map(|_| ())
        .map_err(|e| format!("无法运行内置 Npcap 安装器: {e}"))
}

#[cfg(not(windows))]
pub fn ensure_installed() {}

#[cfg(not(windows))]
pub fn launch_installer() -> Result<(), String> {
    Err("仅支持 Windows".to_string())
}
