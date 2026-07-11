# EQL Meter — Windows

Use this when **EverQuest Legends and EQL Meter both run on Windows** (the normal player setup).

## For players (what you want)

You do **not** need Node, Rust, or Git. Download an installer and run it.

1. Get the latest **Windows installer** (`.exe`) from  
   [Releases](https://github.com/kpxcoolx/eql-meter/releases)  
   (or a draft release your team published from GitHub Actions).
2. Run the NSIS installer — current-user install, **no admin required**.
3. Start **EQL Meter**.
4. Click **Find Legends log** (or **Menu → Monitor → Find Legends log**).
5. Open **Overlay**, then enable **Click-through to game**.

### First-run checklist

1. EQ Legends logging is **on**
2. A recent log exists (see path below)
3. Meter shows **Monitoring** and your character name
4. Overlay is open and set to click-through
5. EQ is **windowed** or **borderless** (not exclusive fullscreen)

## Where the log lives

```text
C:\Users\Public\Daybreak Game Company\Installed Games\EverQuest Legends\Logs
eqlog_<Character>_<Server>.txt
```

Example: `eqlog_Francis_legends.txt`

## Overlay

| Control | What it does |
|---------|----------------|
| **Overlay** button | Open / close the floating meter |
| Click-through | Mouse goes to the game |
| **Ctrl+Shift+U** | Make overlay clickable |
| **Ctrl+Shift+L** | Click-through to game again |

When the EQ client exits, the overlay **auto-hides** on Windows.

## Monitor menu

| Item | Use |
|------|-----|
| Find Legends log | Auto-detect under the public Daybreak path |
| Choose log… | Pick `eqlog_*.txt` yourself |
| Replay whole log… | Parse from the start of the file |
| Resume last log | Re-attach after restart |
| Stop monitoring | Detach |

## Combat / raid tips

- **Copy parse for chat** — compact summary for group/raid chat
- Multi-mob pulls — left rail **Combined** merges live mobs
- In game, run `/who all raid` — then open the **Raid** tab
- **Misc** — loot, randoms, rolls, chat
- **Heals** — healing done + healing received

## Troubleshooting

| Problem | Try |
|---------|-----|
| No log found | Confirm the Logs path; use **Choose log…** |
| Meter idle while fighting | Logging on? **Resume last log** or **Find Legends log** again |
| Overlay blocks the game | **Ctrl+Shift+L** |
| Overlay gone after closing EQ | Expected — reopen when you launch EQ |
| Wrong character | Open the correct `eqlog_YourName_*.txt` |

---

## For contributors only (build from source)

Players should ignore this section. Building requires Node.js 22+, Rust, and several minutes of compile time.

```bash
git clone https://github.com/kpxcoolx/eql-meter.git
cd eql-meter
npm install
npm run tauri:build:windows
```

Installer output: `src-tauri\target\release\bundle\nsis\`

Or run the **windows-build** workflow (Actions → Run workflow) to produce a **draft GitHub Release** with the `.exe` attached.

Dev mode without packaging:

```bash
npm run tauri:dev
```

## Mac / Parallels

If EQ runs in a VM and the meter runs on the Mac host, see [Mac + Parallels](mac-parallels.md).
