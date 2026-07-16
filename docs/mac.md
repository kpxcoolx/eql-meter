# EQL Meter — Mac (osxEQL)

**EQ Legends via [osxEQL](https://github.com/kpxcoolx/osxEQL)** (Wine on Apple Silicon) · **EQL Meter as a native Mac app**

Auto-detect looks in the osxEQL Wine prefix for your character log. No Parallels VM required.

![EQL Meter](images/main-window.png)

---

## Install

1. Download the **`.dmg`** from [GitHub Releases](https://github.com/kpxcoolx/eql-meter/releases/latest).
2. Open it and drag **EQL Meter** into **Applications**.
3. If macOS blocks the first open (“damaged” / can’t be opened), clear quarantine once in Terminal:

```bash
xattr -dr com.apple.quarantine "/Applications/EQL Meter.app"
```

4. You also need EQ Legends running through **osxEQL** (separate app / DMG).

Windows players → [Windows guide](windows.md).  
Still on Parallels? → [Mac + Parallels](mac-parallels.md).

---

## Checklist

- [ ] Apple Silicon Mac (M1 or newer)
- [ ] [osxEQL](https://github.com/kpxcoolx/osxEQL) installed and EQ Legends playable
- [ ] Logging enabled in EQ; you’ve played enough to create `eqlog_*.txt`
- [ ] EQL Meter `.dmg` installed

---

## Attach to the log

1. Launch **osxEQL** and log into EQ Legends  
2. Open **EQL Meter**  
3. Click **Auto-detect**  

Typical path:

```text
~/Library/Application Support/osxEQL/prefix/drive_c/users/Public/Daybreak Game Company/Installed Games/EverQuest Legends/Logs/eqlog_*.txt
```

### If Auto-detect fails

1. Confirm a log exists at the path above (Finder → Go → Go to Folder…)  
2. **Menu → Choose log…** and pick `eqlog_<YourName>_*.txt`  
3. Or **Menu → Live Parallels log** if you actually play in a VM instead

---

## Overlay

| Control | Action |
|---------|--------|
| **Overlay** | Open / close floating meter |
| Click-through (Menu) | Clicks reach the game |
| `Cmd+Shift+U` | Overlay clickable |
| `Cmd+Shift+L` | Click-through to game |

Run EQ **windowed / borderless** so the overlay can sit on top of the Wine window.

---

## Build from source (contributors)

```bash
npm install
npm run tauri:build:mac
```

Output: `src-tauri/target/release/bundle/dmg/`
