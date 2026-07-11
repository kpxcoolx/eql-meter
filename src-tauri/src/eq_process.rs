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
    use std::os::windows::process::CommandExt;

    // GUI apps flash a console for every console-subsystem child unless this
    // flag is set. Without it, the 4s EQ watcher pops a terminal forever.
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let Ok(output) = std::process::Command::new("tasklist")
        .args(["/FO", "CSV", "/NH"])
        .creation_flags(CREATE_NO_WINDOW)
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
