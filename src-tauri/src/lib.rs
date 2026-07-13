mod fight;
mod log_find;
mod log_tail;
mod parse;
mod settings;
mod spells;

use fight::{FightSummary, FightTracker, MeterState};
use log_find::{best_log, best_parallels_log, find_eq_logs, FoundLog};
use log_tail::TailHandle;
use parking_lot::Mutex;
use parse::{character_from_path, detect_stance_from_text, parse_line, server_from_path};
use settings::{
    load_settings, remember_log_path, remember_spells_path, remember_window, save_settings,
    AppSettings, WindowGeometry,
};
use spells::SpellCatalog;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, State, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder, WindowEvent,
};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

static SHUTTING_DOWN: AtomicBool = AtomicBool::new(false);

struct AppState {
    tracker: Mutex<FightTracker>,
    tail: Mutex<Option<TailHandle>>,
    overlay_open: Mutex<bool>,
    overlay_click_through: Mutex<bool>,
    spells: Mutex<SpellCatalog>,
}

#[derive(Clone, serde::Serialize)]
struct OverlayStatus {
    open: bool,
    click_through: bool,
    x: Option<f64>,
    y: Option<f64>,
}

#[derive(Clone, serde::Serialize)]
struct SpellsStatus {
    path: Option<String>,
    count: u64,
}

#[tauri::command]
fn host_os() -> &'static str {
    std::env::consts::OS
}

#[tauri::command]
fn get_settings(app: AppHandle) -> AppSettings {
    load_settings(&app)
}

#[tauri::command]
fn save_app_settings(
    settings: AppSettings,
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
) -> Result<AppSettings, String> {
    // Keep window geometry unless the caller sent newer values.
    let mut merged = load_settings(&app);
    merged.last_log_path = settings.last_log_path;
    merged.auto_monitor_on_start = settings.auto_monitor_on_start;
    merged.focus_primary = settings.focus_primary;
    merged.min_fight_damage = settings.min_fight_damage;
    merged.self_only = settings.self_only;
    if settings.main_window.is_some() {
        merged.main_window = settings.main_window;
    }
    if settings.overlay_window.is_some() {
        merged.overlay_window = settings.overlay_window;
    }
    if settings.spells_path.is_some() {
        merged.spells_path = settings.spells_path;
    }
    save_settings(&app, &merged)?;
    {
        let mut tracker = state.tracker.lock();
        tracker.set_options(
            merged.focus_primary,
            merged.min_fight_damage,
            merged.self_only,
        );
    }
    let _ = emit_state(&state, &app);
    Ok(merged)
}

#[tauri::command]
fn set_self_only(
    enabled: bool,
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
) -> Result<MeterState, String> {
    let mut settings = load_settings(&app);
    settings.self_only = enabled;
    save_settings(&app, &settings)?;
    {
        let mut tracker = state.tracker.lock();
        tracker.set_self_only(enabled);
    }
    Ok(emit_state(&state, &app))
}

#[tauri::command]
fn save_window_geometry(
    label: String,
    geometry: WindowGeometry,
    app: AppHandle,
) -> Result<AppSettings, String> {
    remember_window(&app, &label, geometry)
}

#[tauri::command]
fn find_logs() -> Vec<FoundLog> {
    find_eq_logs()
}

#[tauri::command]
fn auto_detect_log() -> Result<FoundLog, String> {
    best_log().ok_or_else(|| {
        if cfg!(target_os = "macos") {
            "No eqlog_*.txt found. On Mac with Parallels, keep the Windows VM running so C: is mounted under /Volumes, or use Menu → Monitor → Choose log…".to_string()
        } else {
            "No eqlog_*.txt found. Check EverQuest Legends\\Logs, or use Menu → Monitor → Choose log…".to_string()
        }
    })
}

#[tauri::command]
fn auto_detect_parallels_log() -> Result<FoundLog, String> {
    best_parallels_log().ok_or_else(|| {
        "No Parallels EQ log found. Start the Windows VM, confirm C: is mounted (Finder → /Volumes), and check:\nUsers/Public/Daybreak Game Company/Installed Games/EverQuest Legends/Logs".to_string()
    })
}

#[tauri::command]
fn get_meter_state(state: State<'_, Arc<AppState>>) -> MeterState {
    decorate_snapshot(state.tracker.lock().snapshot(), &state)
}

#[tauri::command]
fn get_overlay_status(app: AppHandle, state: State<'_, Arc<AppState>>) -> OverlayStatus {
    status(&app, state.inner())
}

#[tauri::command]
fn reset_fight(state: State<'_, Arc<AppState>>, app: AppHandle) -> MeterState {
    {
        let mut tracker = state.tracker.lock();
        tracker.reset_active();
    }
    emit_state(&state, &app)
}

#[tauri::command]
fn clear_fights(state: State<'_, Arc<AppState>>, app: AppHandle) -> MeterState {
    {
        let mut tracker = state.tracker.lock();
        tracker.clear_all();
    }
    emit_state(&state, &app)
}

#[tauri::command]
fn remove_fight(fight_id: u64, state: State<'_, Arc<AppState>>, app: AppHandle) -> MeterState {
    {
        let mut tracker = state.tracker.lock();
        tracker.remove_fight(fight_id);
    }
    emit_state(&state, &app)
}

#[tauri::command]
fn stop_monitoring(state: State<'_, Arc<AppState>>, app: AppHandle) -> MeterState {
    if let Some(handle) = state.tail.lock().take() {
        handle.stop();
    }
    {
        let mut tracker = state.tracker.lock();
        tracker.set_monitoring(false);
    }
    emit_state(&state, &app)
}

#[tauri::command]
fn start_monitoring(
    path: String,
    from_start: bool,
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
) -> Result<MeterState, String> {
    if let Some(handle) = state.tail.lock().take() {
        handle.stop();
    }

    let path_buf = PathBuf::from(&path);
    if !path_buf.exists() {
        return Err(format!("Log file not found: {path}"));
    }

    let character = character_from_path(&path);
    let server = server_from_path(&path);
    let stance = scan_stance(&path_buf);
    {
        let mut tracker = state.tracker.lock();
        tracker.set_log_path(Some(path.clone()));
        tracker.set_character(character);
        tracker.set_server(server);
        tracker.set_stance(stance);
        tracker.set_monitoring(true);
        if from_start {
            tracker.clear_all();
        }
    }

    let app_state = Arc::clone(&state);
    let app_handle = app.clone();

    let handle = log_tail::start_tailing(path_buf, from_start, move |line| {
        if let Some(event) = parse_line(&line) {
            let event = {
                let spells = app_state.spells.lock();
                spells.enrich_event(event)
            };
            {
                let mut tracker = app_state.tracker.lock();
                tracker.ingest(event);
            }
            let snapshot = decorate_snapshot(app_state.tracker.lock().snapshot(), &app_state);
            let _ = app_handle.emit("meter-update", snapshot);
        }
    })?;

    *state.tail.lock() = Some(handle);
    let _ = remember_log_path(&app, &path);
    Ok(decorate_snapshot(state.tracker.lock().snapshot(), &state))
}

#[tauri::command]
fn ingest_demo_lines(
    lines: Vec<String>,
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
) -> MeterState {
    {
        let mut tracker = state.tracker.lock();
        tracker.set_character(Some("Francis".to_string()));
        tracker.set_log_path(Some("demo".to_string()));
        tracker.set_monitoring(true);
        for line in lines {
            if let Some(event) = parse_line(&line) {
                let event = state.spells.lock().enrich_event(event);
                tracker.ingest(event);
            }
        }
    }
    emit_state(&state, &app)
}

#[tauri::command]
fn get_spells_status(state: State<'_, Arc<AppState>>) -> SpellsStatus {
    let spells = state.spells.lock();
    SpellsStatus {
        path: spells.path.clone(),
        count: spells.len() as u64,
    }
}

#[tauri::command]
fn load_spells_file(
    path: String,
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
) -> Result<SpellsStatus, String> {
    let catalog = SpellCatalog::load(PathBuf::from(&path).as_path())?;
    let status = SpellsStatus {
        path: catalog.path.clone(),
        count: catalog.len() as u64,
    };
    *state.spells.lock() = catalog;
    remember_spells_path(&app, &path)?;
    let _ = emit_state(&state, &app);
    Ok(status)
}

#[tauri::command]
fn clear_spells_file(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
) -> Result<SpellsStatus, String> {
    *state.spells.lock() = SpellCatalog::default();
    let mut settings = load_settings(&app);
    settings.spells_path = None;
    save_settings(&app, &settings)?;
    let _ = emit_state(&state, &app);
    Ok(SpellsStatus {
        path: None,
        count: 0,
    })
}

fn apply_window_geometry(window: &WebviewWindow, geometry: &WindowGeometry) {
    let _ = window.set_position(LogicalPosition::new(geometry.x, geometry.y));
    let _ = window.set_size(LogicalSize::new(
        geometry.width.max(200.0),
        geometry.height.max(40.0),
    ));
}

fn capture_window_geometry(window: &WebviewWindow) -> Option<WindowGeometry> {
    let pos = window.outer_position().ok()?;
    let size = window.outer_size().ok()?;
    let scale = window.scale_factor().unwrap_or(1.0);
    if scale <= 0.0 {
        return None;
    }
    Some(WindowGeometry {
        x: f64::from(pos.x) / scale,
        y: f64::from(pos.y) / scale,
        width: f64::from(size.width) / scale,
        height: f64::from(size.height) / scale,
    })
}

fn persist_window_geometry(app: &AppHandle, label: &str) {
    let Some(window) = app.get_webview_window(label) else {
        return;
    };
    let Some(geometry) = capture_window_geometry(&window) else {
        return;
    };
    // Never remember off-screen coords (disconnected display / Parallels shuffle).
    if !geometry_visible_on_any_monitor(app, &geometry) {
        return;
    }
    let _ = remember_window(app, label, geometry);
}

fn geometry_visible_on_any_monitor(app: &AppHandle, geometry: &WindowGeometry) -> bool {
    let Ok(monitors) = app.available_monitors() else {
        return true;
    };
    if monitors.is_empty() {
        return true;
    }
    let cx = geometry.x + geometry.width * 0.5;
    let cy = geometry.y + geometry.height * 0.5;
    for monitor in monitors {
        let scale = monitor.scale_factor();
        if scale <= 0.0 {
            continue;
        }
        let pos = monitor.position();
        let size = monitor.size();
        let left = f64::from(pos.x) / scale;
        let top = f64::from(pos.y) / scale;
        let right = left + f64::from(size.width) / scale;
        let bottom = top + f64::from(size.height) / scale;
        if cx >= left && cx < right && cy >= top && cy < bottom {
            return true;
        }
    }
    false
}

fn place_overlay_on_screen(app: &AppHandle, geometry: &WindowGeometry) -> WindowGeometry {
    let width = geometry.width.max(300.0);
    let height = geometry.height.max(54.0);
    if geometry_visible_on_any_monitor(app, geometry) {
        return WindowGeometry {
            x: geometry.x,
            y: geometry.y,
            width,
            height,
        };
    }

    // Saved position is off every display — park near the primary top-right.
    if let Ok(Some(monitor)) = app.primary_monitor() {
        let scale = monitor.scale_factor().max(0.1);
        let pos = monitor.position();
        let size = monitor.size();
        let left = f64::from(pos.x) / scale;
        let top = f64::from(pos.y) / scale;
        let monitor_w = f64::from(size.width) / scale;
        return WindowGeometry {
            x: left + monitor_w - width - 24.0,
            y: top + 48.0,
            width,
            height,
        };
    }

    WindowGeometry {
        x: 80.0,
        y: 80.0,
        width,
        height,
    }
}

#[tauri::command]
async fn show_overlay(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<OverlayStatus, String> {
    // async is required on Windows — sync WebviewWindowBuilder::build from a
    // command blanks / freezes the secondary WebView2.
    show_overlay_inner(&app, state.inner())
}

fn show_overlay_inner(app: &AppHandle, state: &Arc<AppState>) -> Result<OverlayStatus, String> {
    // Always open in setup mode (clickable) so the user can drag/position.
    *state.overlay_click_through.lock() = false;

    if let Some(window) = app.get_webview_window("overlay") {
        if let Some(current) = capture_window_geometry(&window) {
            if !geometry_visible_on_any_monitor(app, &current) {
                let safe = place_overlay_on_screen(app, &current);
                apply_window_geometry(&window, &safe);
                let _ = remember_window(app, "overlay", safe);
            }
        }
        let _ = window.set_always_on_top(true);
        window
            .set_ignore_cursor_events(false)
            .map_err(|e| e.to_string())?;
        // Reload so a previously blank WebView2 gets a fresh document.
        let _ = window.eval(
            "if (!document.documentElement.dataset.eqlReady) { location.reload(); }",
        );
        window.show().map_err(|e| e.to_string())?;
        let _ = window.set_focus();
        *state.overlay_open.lock() = true;
        emit_overlay_status(app, state);
        return Ok(status(app, state));
    }

    let window = create_overlay_window(app)?;
    let _ = window.set_always_on_top(true);
    let _ = window.set_ignore_cursor_events(false);
    let _ = window.show();
    let _ = window.set_focus();

    *state.overlay_open.lock() = true;
    emit_overlay_status(app, state);
    Ok(status(app, state))
}

fn create_overlay_window(app: &AppHandle) -> Result<tauri::WebviewWindow, String> {
    let settings = load_settings(app);
    let mut builder =
        WebviewWindowBuilder::new(app, "overlay", WebviewUrl::App("overlay.html".into()))
            .title("EQL Overlay")
            .inner_size(380.0, 120.0)
            .min_inner_size(300.0, 54.0)
            .resizable(false)
            .decorations(false)
            .always_on_top(true)
            .visible_on_all_workspaces(true)
            .skip_taskbar(true)
            .focused(false)
            // Start hidden; show after create so Windows does not flash a dead surface.
            .visible(false);

    #[cfg(target_os = "windows")]
    {
        builder = builder.drag_and_drop(false);
    }

    #[cfg(target_os = "windows")]
    {
        builder = builder
            .transparent(false)
            .shadow(false)
            .background_color(tauri::window::Color(18, 16, 14, 255));
    }
    #[cfg(not(target_os = "windows"))]
    {
        builder = builder
            .transparent(true)
            .shadow(false)
            .background_color(tauri::window::Color(0, 0, 0, 0));
    }

    let seed = settings.overlay_window.clone().unwrap_or(WindowGeometry {
        x: 80.0,
        y: 80.0,
        width: 380.0,
        height: 120.0,
    });
    let safe = place_overlay_on_screen(app, &seed);
    builder = builder
        .position(safe.x, safe.y)
        .inner_size(safe.width.max(300.0), safe.height.max(100.0));

    let window = builder.build().map_err(|e| e.to_string())?;

    #[cfg(target_os = "windows")]
    let _ = window.set_background_color(Some(tauri::window::Color(18, 16, 14, 255)));
    #[cfg(not(target_os = "windows"))]
    let _ = window.set_background_color(Some(tauri::window::Color(0, 0, 0, 0)));

    apply_window_geometry(&window, &safe);
    let _ = remember_window(app, "overlay", safe);
    Ok(window)
}

fn hide_overlay_inner(app: &AppHandle, state: &Arc<AppState>) -> Result<OverlayStatus, String> {
    if let Some(window) = app.get_webview_window("overlay") {
        if let Some(geometry) = capture_window_geometry(&window) {
            let _ = remember_window(app, "overlay", geometry);
        }
        let _ = window.set_ignore_cursor_events(false);
        // Hide instead of close — recreating WebView2 from a command is flaky on Windows.
        window.hide().map_err(|e| e.to_string())?;
    }
    *state.overlay_open.lock() = false;
    *state.overlay_click_through.lock() = false;
    emit_overlay_status(app, state);
    Ok(status(app, state))
}

#[tauri::command]
async fn hide_overlay(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<OverlayStatus, String> {
    hide_overlay_inner(&app, state.inner())
}

#[tauri::command]
async fn toggle_overlay(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<OverlayStatus, String> {
    let window_open = app
        .get_webview_window("overlay")
        .map(|w| w.is_visible().unwrap_or(false))
        .unwrap_or(false);
    if window_open || *state.overlay_open.lock() {
        hide_overlay_inner(&app, state.inner())
    } else {
        show_overlay_inner(&app, state.inner())
    }
}

#[tauri::command]
fn set_overlay_click_through(
    enabled: bool,
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<OverlayStatus, String> {
    let window = app
        .get_webview_window("overlay")
        .ok_or_else(|| "Overlay is not open".to_string())?;

    window
        .set_ignore_cursor_events(enabled)
        .map_err(|e| e.to_string())?;
    *state.overlay_click_through.lock() = enabled;

    if !enabled {
        let _ = window.set_focus();
    }

    emit_overlay_status(&app, &state);
    Ok(status(&app, &state))
}

#[tauri::command]
fn copy_parse(
    fight_id: Option<u64>,
    fight_ids: Option<Vec<u64>>,
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<String, String> {
    let tracker = state.tracker.lock();
    let text = if let Some(ids) = fight_ids.as_ref().filter(|ids| !ids.is_empty()) {
        tracker
            .format_parse_ids(ids)
            .ok_or_else(|| "No fight to copy".to_string())?
    } else {
        tracker
            .format_parse(fight_id)
            .ok_or_else(|| "No fight to copy".to_string())?
    };
    drop(tracker);
    app.clipboard()
        .write_text(&text)
        .map_err(|e| e.to_string())?;
    Ok(text)
}

#[tauri::command]
fn combine_fights(
    fight_ids: Vec<u64>,
    state: State<'_, Arc<AppState>>,
) -> Result<FightSummary, String> {
    state
        .tracker
        .lock()
        .combine_by_ids(&fight_ids)
        .ok_or_else(|| "Select at least one fight to combine".to_string())
}

fn apply_click_through(
    app: &AppHandle,
    state: &Arc<AppState>,
    enabled: bool,
) -> Result<(), String> {
    *state.overlay_click_through.lock() = enabled;
    if let Some(window) = app.get_webview_window("overlay") {
        window
            .set_ignore_cursor_events(enabled)
            .map_err(|e| e.to_string())?;
        if !enabled {
            let _ = window.set_focus();
        }
    }
    emit_overlay_status(app, state);
    let _ = app.emit(
        "hotkey-toast",
        if enabled {
            "Overlay click-through ON (Ctrl+Shift+U to unlock)"
        } else {
            "Overlay unlocked (Ctrl+Shift+L to lock)"
        },
    );
    Ok(())
}

fn status(app: &AppHandle, state: &Arc<AppState>) -> OverlayStatus {
    let window = app.get_webview_window("overlay");
    let open = window
        .as_ref()
        .map(|w| w.is_visible().unwrap_or(false))
        .unwrap_or(false);
    *state.overlay_open.lock() = open;
    let click_through = *state.overlay_click_through.lock();
    let mut x = None;
    let mut y = None;
    if let Some(window) = window.as_ref() {
        if let Some(geo) = capture_window_geometry(window) {
            x = Some(geo.x);
            y = Some(geo.y);
        }
    }
    OverlayStatus {
        open,
        click_through,
        x,
        y,
    }
}

fn shutdown_app(app: &AppHandle) {
    if SHUTTING_DOWN.swap(true, Ordering::SeqCst) {
        return;
    }

    persist_window_geometry(app, "main");
    if let Some(state) = app.try_state::<Arc<AppState>>() {
        if let Some(handle) = state.tail.lock().take() {
            handle.stop();
        }
        let _ = hide_overlay_inner(app, state.inner());
    }
    // Destroy any leftover windows (overlay can keep the process alive on Windows).
    for (label, window) in app.webview_windows() {
        if label != "main" {
            let _ = window.destroy();
        }
    }
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.destroy();
    }
    let _ = app.global_shortcut().unregister_all();
    app.exit(0);
    // Hard fallback — if the event loop stalls (common with WebView2 + extra windows),
    // still leave Task Manager clean.
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(800));
        std::process::exit(0);
    });
}

fn emit_overlay_status(app: &AppHandle, state: &Arc<AppState>) {
    let _ = app.emit("overlay-status", status(app, state));
}

fn emit_state(state: &Arc<AppState>, app: &AppHandle) -> MeterState {
    let snapshot = decorate_snapshot(state.tracker.lock().snapshot(), state);
    let _ = app.emit("meter-update", &snapshot);
    snapshot
}

fn decorate_snapshot(mut snapshot: MeterState, state: &AppState) -> MeterState {
    let spells = state.spells.lock();
    snapshot.spells_count = spells.len() as u64;
    snapshot.spells_path = spells.path.clone();
    snapshot
}

fn try_load_spells(state: &AppState, path: &str) {
    match SpellCatalog::load(PathBuf::from(path).as_path()) {
        Ok(catalog) => {
            *state.spells.lock() = catalog;
        }
        Err(err) => {
            eprintln!("spells_us load failed ({path}): {err}");
        }
    }
}

/// Read the end of the log (or whole file if small) to recover active
/// stance after attaching mid-session.
fn scan_stance(path: &PathBuf) -> Option<String> {
    let Ok(meta) = std::fs::metadata(path) else {
        return None;
    };
    let len = meta.len();
    let read_from = len.saturating_sub(512_000);
    let Ok(mut file) = std::fs::File::open(path) else {
        return None;
    };
    use std::io::{Read, Seek, SeekFrom};
    if file.seek(SeekFrom::Start(read_from)).is_err() {
        return None;
    }
    let mut buf = String::new();
    if file.read_to_string(&mut buf).is_err() {
        return None;
    }
    detect_stance_from_text(&buf)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = Arc::new(AppState {
        tracker: Mutex::new(FightTracker::default()),
        tail: Mutex::new(None),
        overlay_open: Mutex::new(false),
        overlay_click_through: Mutex::new(false),
        spells: Mutex::new(SpellCatalog::default()),
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(state.clone())
        .on_window_event(|window, event| {
            match event {
                WindowEvent::Moved(_) | WindowEvent::Resized(_) => {
                    let label = window.label().to_string();
                    if label == "main" || label == "overlay" {
                        persist_window_geometry(window.app_handle(), &label);
                    }
                }
                WindowEvent::CloseRequested { api, .. } => {
                    // Overlay is a second window. Closing main must fully quit —
                    // otherwise Windows leaves eql-meter.exe alive in Task Manager.
                    if window.label() != "main" {
                        return;
                    }
                    api.prevent_close();
                    shutdown_app(window.app_handle());
                }
                WindowEvent::Destroyed => {
                    if window.label() == "main" {
                        // Backup path if close skipped CloseRequested.
                        shutdown_app(window.app_handle());
                        return;
                    }
                    if window.label() != "overlay" {
                        return;
                    }
                    let Some(app_state) = window.app_handle().try_state::<Arc<AppState>>() else {
                        return;
                    };
                    *app_state.overlay_open.lock() = false;
                    *app_state.overlay_click_through.lock() = false;
                    emit_overlay_status(window.app_handle(), app_state.inner());
                }
                _ => {}
            }
        })
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state != tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        return;
                    }
                    let Some(app_state) = app.try_state::<Arc<AppState>>() else {
                        return;
                    };
                    match shortcut.key {
                        tauri_plugin_global_shortcut::Code::KeyU => {
                            let _ = apply_click_through(app, app_state.inner(), false);
                        }
                        tauri_plugin_global_shortcut::Code::KeyL => {
                            let _ = apply_click_through(app, app_state.inner(), true);
                        }
                        _ => {}
                    }
                })
                .build(),
        )
        .setup(|app| {
            use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

            let settings = load_settings(app.handle());
            if let Some(app_state) = app.try_state::<Arc<AppState>>() {
                app_state
                    .tracker
                    .lock()
                    .set_options(
                        settings.focus_primary,
                        settings.min_fight_damage,
                        settings.self_only,
                    );
                if let Some(ref spells_path) = settings.spells_path {
                    try_load_spells(app_state.inner(), spells_path);
                }
            }

            if let Some(geo) = settings.main_window.as_ref() {
                if let Some(main) = app.get_webview_window("main") {
                    apply_window_geometry(&main, geo);
                }
            }

            #[cfg(target_os = "macos")]
            let mods = Modifiers::SUPER | Modifiers::SHIFT;
            #[cfg(not(target_os = "macos"))]
            let mods = Modifiers::CONTROL | Modifiers::SHIFT;

            app.global_shortcut()
                .register(Shortcut::new(Some(mods), Code::KeyU))
                .map_err(|e| e.to_string())?;
            app.global_shortcut()
                .register(Shortcut::new(Some(mods), Code::KeyL))
                .map_err(|e| e.to_string())?;

            // Create overlay on the main thread at startup (hidden). Creating it later
            // from a sync IPC command blanks WebView2 on Windows.
            if let Err(err) = create_overlay_window(app.handle()) {
                eprintln!("overlay pre-create failed: {err}");
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_meter_state,
            get_overlay_status,
            host_os,
            get_settings,
            save_app_settings,
            set_self_only,
            save_window_geometry,
            find_logs,
            auto_detect_log,
            auto_detect_parallels_log,
            start_monitoring,
            stop_monitoring,
            reset_fight,
            clear_fights,
            remove_fight,
            ingest_demo_lines,
            get_spells_status,
            load_spells_file,
            clear_spells_file,
            show_overlay,
            hide_overlay,
            toggle_overlay,
            set_overlay_click_through,
            copy_parse,
            combine_fights
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
