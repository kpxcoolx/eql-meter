# EQL Meter — Mac + Parallels

Use this when **EverQuest Legends runs in a Windows VM** (Parallels) and **EQL Meter runs on the Mac**.

The meter reads the Windows log file through the mounted `C:` volume under `/Volumes`.

## No Mac installer

There is **no macOS `.dmg` (or other Mac package)** on GitHub Releases. Releases only ship the **Windows** NSIS `.exe`.

On Mac you always run from source (`npm run tauri:dev` below). That is the supported Mac path for Parallels dogfooding — not a missing download.

Windows players should use the installer: [Windows](windows.md).

## Requirements

- Parallels Desktop with a Windows VM
- EverQuest Legends installed in that VM
- Logging enabled in EQ Legends
- Node.js 22+ and Rust (to build and run the meter locally)

## One-time setup

1. Start the Windows VM.
2. Confirm the Windows disk is mounted:
   - Finder → **Go → Go to Folder…** → `/Volumes`
   - You should see something like `[C] Windows 11` or `[C] Windows 11.hidden`
3. In EQ Legends (inside the VM), turn **logging** on if it is not already.
4. Play briefly so a log file exists under:

   `C:\Users\Public\Daybreak Game Company\Installed Games\EverQuest Legends\Logs`

   File name looks like: `eqlog_YourName_legends.txt`

## Run the meter on Mac

Clone and run in dev mode (this is how you get the app on Mac):

```bash
git clone https://github.com/kpxcoolx/eql-meter.git
cd eql-meter
npm install
npm run tauri:dev
```

There is no published Mac build step to skip this.

## Attach to the live log

1. Keep the Windows VM running (so `C:` stays mounted).
2. In EQL Meter, click **Live Parallels** (top bar), or **Menu → Monitor → Live Parallels log**.
3. Play in EQ — combat should update the meter within about ~150ms.

### If Live Parallels fails

1. Check `/Volumes` again — the VM disk must be mounted.
2. Use **Menu → Monitor → Choose log…** and pick the file manually. Typical Mac path:

   `/Volumes/[C] Windows 11.hidden/Users/Public/Daybreak Game Company/Installed Games/EverQuest Legends/Logs/eqlog_*.txt`

3. Or use **Menu → Monitor → Find any eqlog…** to search mounted volumes.

## Overlay on Mac

- Click **Overlay** to open the floating meter.
- **Menu → Overlay → Click-through to game** so mouse clicks reach EQ in the VM window.
- **Cmd+Shift+U** — make overlay clickable again  
- **Cmd+Shift+L** — click-through to game  

Run EQ windowed or borderless inside the VM so you can see the Mac overlay over (or beside) the game window.

## Useful Monitor menu items

| Item | Use |
|------|-----|
| Live Parallels log | Best default while the VM is up |
| Find any eqlog… | Search for any `eqlog_*.txt` on mounted disks |
| Choose log… | Pick a file yourself |
| Replay whole log… | Parse the file from the start (history) |
| Resume last log | Re-attach to the path you used last time |
| Stop monitoring | Detach from the log |

## Tips

- If the meter goes quiet after a VM sleep/resume, click **Live Parallels** again or **Resume last log**.
- Parallels mounts do not always fire reliable file events — the meter polls frequently so appends still show up.
- Sample fights: **Menu → Extras → Load sample fight** (no game required).

## Not for Mac-only play

EQL Meter does not replace a Windows install of EQ. On Mac you are always reading a **Windows** log (via Parallels or a copied file). For playing on a Windows PC, see [Windows](windows.md).
