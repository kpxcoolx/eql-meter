# EQL Meter — Mac + Parallels

**EQ Legends in a Windows VM** · **EQL Meter on the Mac host**

Prefer native Wine? Use [osxEQL + Mac DMG](mac.md) instead — Auto-detect finds that log path without a VM.

The meter can still read the Windows log through Parallels’ mounted `C:` under `/Volumes`.

![EQL Meter](images/main-window.png)

---

## Install

1. Download the **macOS `.dmg`** from [GitHub Releases](https://github.com/kpxcoolx/eql-meter/releases/latest).
2. Drag **EQL Meter** into **Applications** (clear quarantine if Gatekeeper blocks the first open — see [Mac guide](mac.md)).
3. Keep your Parallels Windows VM running so `C:` stays mounted.

Or run from source:

```bash
git clone https://github.com/kpxcoolx/eql-meter.git
cd eql-meter
npm install
npm run tauri:dev
```

Windows-only players → [Windows guide](windows.md).

---

## Checklist

- [ ] Parallels Windows VM running
- [ ] EQ Legends + logging on inside the VM
- [ ] Windows disk visible under `/Volumes` (e.g. `[C] Windows 11` or `[C] Windows 11.hidden`)
- [ ] EQL Meter installed (DMG) or running via `tauri:dev`

---

## 1. Mount + log file

1. Start the Windows VM  
2. Finder → **Go → Go to Folder…** → `/Volumes`  
3. In EQ, enable logging and play briefly so a log exists:

```text
C:\Users\Public\Daybreak Game Company\Installed Games\EverQuest Legends\Logs
eqlog_<YourName>_*.txt
```

---

## 2. Attach to the log

1. Keep the VM running (so `C:` stays mounted)  
2. Click **Auto-detect**, or **Menu → Live Parallels log**  
3. Fight in EQ — the meter should update within ~150ms  

### If detection fails

1. Confirm `/Volumes` still shows the Windows disk  
2. **Menu → Choose log…** — typical path:

```text
/Volumes/[C] Windows 11.hidden/Users/Public/Daybreak Game Company/Installed Games/EverQuest Legends/Logs/eqlog_*.txt
```

---

## Overlay

| Control | Action |
|---------|--------|
| **Overlay** | Open / close floating meter |
| Click-through (Menu) | Clicks reach EQ in the VM |
| `Cmd+Shift+U` | Overlay clickable |
| `Cmd+Shift+L` | Click-through to game |

Run EQ **windowed / borderless** in the VM so you can see the Mac overlay.

---

## Tips

| Situation | What to do |
|-----------|------------|
| Quiet after VM sleep | **Auto-detect**, **Live Parallels log**, or **Resume last log** again |
| Share a parse | **Copy Parse** |
| No game handy | Load a sample log from [`samples/`](../samples/) if available |

Parallels mounts are not always perfect for file events — the meter polls so appends still show up.
