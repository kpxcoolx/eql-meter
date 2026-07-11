# EQL Meter — Windows

**EverQuest Legends + EQL Meter on the same Windows PC** (normal player setup).

![EQL Meter](images/main-window.png)

---

## Install (players)

You do **not** need Node, Rust, or Git.

1. Download **EQL.Meter_\*_x64-setup.exe** from  
   [Latest release](https://github.com/kpxcoolx/eql-meter/releases/latest)
2. Run the installer (current-user; **no admin**)
3. Start **EQL Meter**
4. Click **Find Legends log** (or **Menu → Monitor → Find Legends log**)
5. Open **Overlay** — confirm the popup, then enable click-through if you need clicks to reach the game

### Ready when…

- [ ] EQ logging is **on**
- [ ] Meter shows **Monitoring** + your character name
- [ ] Overlay opened (confirmation popup) and click-through is on if you want
- [ ] EQ is **windowed** or **borderless** (not exclusive fullscreen)

---

## Log path

```text
C:\Users\Public\Daybreak Game Company\Installed Games\EverQuest Legends\Logs
eqlog_<Character>_<Server>.txt
```

Example: `eqlog_Francis_legends.txt`

---

## Overlay

| Control | What it does |
|---------|----------------|
| **Overlay** button | Open / close (shows a confirmation with screen position) |
| Click-through | Mouse reaches the game |
| `Ctrl+Shift+U` | Make overlay clickable |
| `Ctrl+Shift+L` | Click-through again |

EQ must be **windowed** or **borderless** (not exclusive fullscreen) for the overlay to appear above the game. On Windows the overlay uses a dark opaque bar so WebView2 stays visible.

---

## Updates

EQL Meter checks GitHub for a newer release on startup.

| Want… | Do this |
|-------|---------|
| See if an update exists | **Menu → Check for updates** |
| Install it | Banner **Install update**, or **Menu → Install …** |
| Manual download | **Menu → Open latest release…** or [Latest release](https://github.com/kpxcoolx/eql-meter/releases/latest) |

The updater reads `latest.json` from the **Latest** GitHub release. CI only publishes a release after the installer + `latest.json` are attached, so Check for updates should never hit an empty release.

---

## Monitor menu

| Item | When to use it |
|------|----------------|
| **Find Legends log** | Default — auto-detect under the Daybreak path |
| **Choose log…** | Pick `eqlog_*.txt` yourself |
| **Replay whole log…** | Parse from the start of the file |
| **Resume last log** | Re-attach after restart |
| **Stop monitoring** | Detach |

---

## During a fight

| Want… | Do this |
|-------|---------|
| Share numbers in chat | **Copy Parse** (confirmation popup with preview) |
| Multi-mob pull | Select **Combined** (or Cmd/Ctrl+click fights) |
| Raid roster / group DPS | In game: `/who all raid` → **Raid** tab |
| Heals | **Heals** tab |
| Loot | **Loot** tab |
| Remove a bad fight | Right-click it in the Fights list → **Delete fight** |

---

## Troubleshooting

| Problem | Fix |
|---------|-----|
| No log found | Confirm the path above → **Choose log…** |
| Idle while fighting | Logging on? **Find Legends log** / **Resume last log** |
| Overlay blocks clicks | `Ctrl+Shift+L` |
| Overlay not visible | Use windowed/borderless EQ; note the position in the confirmation popup; update to **v0.1.8+** |
| Copy Parse did nothing | You should get a confirmation popup — if you see an error, update or report it |
| Pet name in the Fights list | Update to **v0.1.8+**; or right-click → **Delete fight** |
| Terminal windows flashing | Update to the latest build; also fully quit — closing main now exits the overlay too |
| Overlay stays after closing app | Update — closing the main window now closes the overlay and exits |
| Wrong character | Open the matching `eqlog_YourName_*.txt` |

---

## Mac / Parallels?

EQ in a VM, meter on the Mac host → [Mac + Parallels](mac-parallels.md)  
(There is **no** Mac `.dmg`.)

---

<details>
<summary><strong>Contributors — build from source</strong></summary>

Players can ignore this. Needs Node.js 22+, Rust, and several minutes to compile.

```bash
git clone https://github.com/kpxcoolx/eql-meter.git
cd eql-meter
npm install
npm run tauri:build:windows
```

Installer: `src-tauri\target\release\bundle\nsis\`

Or run **windows-build** in GitHub Actions to attach a draft release `.exe`.

Dev mode:

```bash
npm run tauri:dev
```

</details>
