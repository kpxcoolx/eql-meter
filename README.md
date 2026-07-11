# EQL Meter

Real-time combat log meter for **EverQuest Legends**.

Develop on macOS. Ship and run on **Windows** next to the game client — that is how players will use it.

## What it does

- Auto-detects `eqlog_<Character>_<Server>.txt` under the EverQuest Legends `Logs` folder
- Remembers the last log and can resume monitoring on launch
- Live fight tracking, group DPS, ability breakdown, damage-over-time chart
- Raid roster from `/who all raid`, plus Misc tab for loot / randoms / chat
- Optional ability-name file (Menu → Extras) if logs show spell IDs instead of names
- In-game overlay with click-through lock
- Opens the log with Windows share-mode so EQ can keep writing
- Remembers main + overlay window positions between launches

## Live logs from Mac + Parallels

1. Start the Windows VM (so C: appears under `/Volumes`)
2. Run `npm run tauri:dev`
3. Click **Live Parallels** (or Menu → Log → Live Parallels log)

That tails:
`/Volumes/[C] Windows 11.hidden/Users/Public/.../EverQuest Legends/Logs/eqlog_*.txt`

Play in EQ — new combat lines should update the meter within ~150ms.

If it fails to find the file: open Finder → Go → Go to Folder → `/Volumes` and confirm the Windows disk is mounted. You can also **Menu → Monitor → Choose log…** and pick the `eqlog_*.txt` manually.
## Ship for Windows

### On a Windows machine

```bash
npm install
npm run tauri:build:windows
```

NSIS installer output:

`src-tauri/target/release/bundle/nsis/`

Install with the generated `.exe` (current-user install, no admin required). Window and overlay positions are remembered between launches.

### Via GitHub Actions

Push a tag like `v0.1.0` or run the **windows-build** workflow manually. It builds an NSIS installer on `windows-latest`.

Players need:

1. Logging enabled in EQ Legends
2. Logs under  
   `C:\Users\Public\Daybreak Game Company\Installed Games\EverQuest Legends\Logs`
3. **Auto-detect** (or Open & Monitor) while playing

On Windows, the overlay auto-hides when the game client is not running.

## In-game overlay

- **Overlay** — floating always-on-top DPS meter
- **Lock / Click-through** — mouse clicks pass through to the game
- **Unlock Overlay** in the main window, or **Ctrl+Shift+U** (Cmd+Shift+U on Mac)
- **Ctrl+Shift+L** / **Cmd+Shift+L** locks click-through again
- **Copy Parse** — clipboard summary for group chat

Use EQ in windowed or borderless mode on Windows.

## Hotkeys

| Shortcut | Action |
|----------|--------|
| Ctrl/Cmd+Shift+U | Unlock overlay (disable click-through) |
| Ctrl/Cmd+Shift+L | Lock overlay (enable click-through) |
