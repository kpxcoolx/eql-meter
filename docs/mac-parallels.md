# EQL Meter — Mac + Parallels

**EQ Legends in a Windows VM** · **EQL Meter on the Mac host**

The meter reads the Windows log through Parallels’ mounted `C:` under `/Volumes`.

![EQL Meter](images/main-window.png)

---

## Important: no Mac installer

| | |
|--|--|
| **GitHub Releases** | Windows `.exe` only |
| **macOS `.dmg`** | Does not exist |
| **How you run on Mac** | Clone + `npm run tauri:dev` (below) |

This is intentional for Parallels dogfooding — not a missing download.  
Windows-only players → [Windows guide](windows.md).

---

## Checklist

- [ ] Parallels Windows VM running
- [ ] EQ Legends + logging on inside the VM
- [ ] Node.js 22+ and Rust on the Mac
- [ ] Windows disk visible under `/Volumes` (e.g. `[C] Windows 11` or `[C] Windows 11.hidden`)

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

## 2. Run the meter

```bash
git clone https://github.com/kpxcoolx/eql-meter.git
cd eql-meter
npm install
npm run tauri:dev
```

There is no packaged Mac build to download instead.

---

## 3. Attach to the log

1. Keep the VM running (so `C:` stays mounted)  
2. Click **Live Parallels** (top bar), or **Menu → Monitor → Live Parallels log**  
3. Fight in EQ — the meter should update within ~150ms  

### If Live Parallels fails

1. Confirm `/Volumes` still shows the Windows disk  
2. **Menu → Monitor → Choose log…** — typical path:

```text
/Volumes/[C] Windows 11.hidden/Users/Public/Daybreak Game Company/Installed Games/EverQuest Legends/Logs/eqlog_*.txt
```

3. Or **Menu → Monitor → Find any eqlog…**

---

## Overlay

| Control | Action |
|---------|--------|
| **Overlay** | Open floating meter |
| **Menu → Overlay → Click-through** | Clicks reach EQ in the VM |
| `Cmd+Shift+U` | Overlay clickable |
| `Cmd+Shift+L` | Click-through to game |

Run EQ **windowed / borderless** in the VM so you can see the Mac overlay.

---

## Monitor menu (quick ref)

| Item | Use |
|------|-----|
| **Live Parallels log** | Best default while the VM is up |
| **Find any eqlog…** | Search mounted disks |
| **Choose log…** | Pick the file yourself |
| **Replay whole log…** | Parse from the start |
| **Resume last log** | Re-attach after restart |
| **Stop monitoring** | Detach |

---

## Tips

| Situation | What to do |
|-----------|------------|
| Quiet after VM sleep | **Live Parallels** or **Resume last log** again |
| Bad / pet-named fight in the list | Right-click → **Delete fight** |
| No game handy | **Menu → Extras → Load sample fight** |

Parallels mounts are not always perfect for file events — the meter polls so appends still show up.

---

## Not for Mac-only play

EQL Meter always reads a **Windows** log (Parallels mount or a copied file). It does not replace running EQ on Windows. Pure Windows setup → [Windows](windows.md).
