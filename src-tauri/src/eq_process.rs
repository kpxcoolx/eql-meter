/// Detect whether an EverQuest / EQ Legends client process is running.
/// Used on Windows to auto-hide the overlay when the game exits.

#[cfg(target_os = "windows")]
const EQ_PROCESS_NAMES: &[&str] = &[
    "eqgame.exe",
    "everquest.exe",
    "everquest legends.exe",
    "eql.exe",
];

#[cfg(target_os = "windows")]
pub fn is_eq_running() -> bool {
    let Ok(output) = std::process::Command::new("tasklist")
        .args(["/FO", "CSV", "/NH"])
        .output()
    else {
        return false;
    };
    let Ok(text) = String::from_utf8(output.stdout) else {
        return false;
    };
    let lower = text.to_ascii_lowercase();
    EQ_PROCESS_NAMES.iter().any(|name| lower.contains(name))
}

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
pub fn is_eq_running() -> bool {
    // Mac/Linux: EQ usually runs in a VM; don't auto-hide based on host processes.
    true
}
