# EQL Meter

Real-time combat meter for **EverQuest Legends**.

It tails your character log (`eqlog_<Character>_<Server>.txt`), tracks live fights, and shows group DPS in a main window plus an always-on-top overlay.

**Repo:** [github.com/kpxcoolx/eql-meter](https://github.com/kpxcoolx/eql-meter)

## Which guide do I need?

| You are… | Guide |
|----------|--------|
| **Playing on Windows** (normal) | [Windows](docs/windows.md) — download the `.exe` installer |
| Developing on **Mac + Parallels** | [Mac + Parallels](docs/mac-parallels.md) |

Players on Windows should **never** need to clone the repo or run `npm`. That path is for contributors building the installer.

## Features

- Live fight tracking, multi-mob Combined view, ability breakdown, DPS chart
- Floating overlay with click-through (clicks pass through to the game)
- Raid roster from `/who all raid`, plus Misc tab for loot / randoms / chat
- Heals tab with healing done and healing received
- Group DPS bars when a raid roster is known
- Remembers last log and window / overlay positions
- Optional ability-name file if logs show spell IDs instead of names
- Windows: overlay auto-hides when the game client is not running

## Menu (quick map)

| Section | What it does |
|---------|----------------|
| **Monitor** | Find / choose / resume / stop the character log |
| **Overlay** | Open/close overlay; click-through to game |
| **Combat** | Copy parse, end fight, clear history, skip tiny fights |
| **Extras** | Optional ability names; load sample fight |

## Overlay hotkeys

| Shortcut | Action |
|----------|--------|
| Ctrl/Cmd+Shift+U | Make overlay clickable |
| Ctrl/Cmd+Shift+L | Click-through to game |

Use EQ in **windowed** or **borderless** mode so the overlay can sit on top.

## Develop (contributors)

```bash
npm install
npm run tauri:dev
```

- **macOS:** [Mac + Parallels](docs/mac-parallels.md) to attach to the VM log.
- **Windows:** [Windows](docs/windows.md) — use **Find Legends log**.

### Ship a Windows installer for players

Do **not** ask players to build. Produce an NSIS `.exe` and put it on GitHub Releases:

1. GitHub → **Actions** → **windows-build** → **Run workflow** (enter `v0.1.0` or similar), **or**
2. `git tag v0.1.0 && git push origin v0.1.0`

The workflow builds on `windows-latest` and attaches the installer to a **draft release**. Publish the draft when you are ready to dogfood.

Local build (on a Windows machine) if you prefer:

```bash
npm install
npm run tauri:build:windows
```

Output: `src-tauri/target/release/bundle/nsis/`

## Sample logs

Parser fixtures live under [`samples/`](samples/). Useful for offline testing without the game running.
