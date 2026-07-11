# EQL Meter

Live combat meter for **EverQuest Legends** — tails your character log, tracks fights in real time, and shows DPS in a main window plus a click-through overlay.

![EQL Meter](docs/images/main-window.png)

[Latest release](https://github.com/kpxcoolx/eql-meter/releases/latest) · [Discussions](https://github.com/kpxcoolx/eql-meter/discussions) · [Changelog](CHANGELOG.md) · [MIT License](LICENSE.md)

---

## Start here

| I am… | Do this |
|-------|---------|
| Playing on **Windows** | Install the `.exe` → [Windows guide](docs/windows.md) |
| On **Mac + Parallels** | No Mac installer — run from source → [Mac guide](docs/mac-parallels.md) |

> **Players:** download the installer. You do not need Node, Rust, or Git.  
> Releases ship **Windows only** — there is no macOS `.dmg`.

---

## What you get

| | |
|--|--|
| **Live fights** | Multi-mob Combined view, ability breakdown, DPS chart |
| **Overlay** | Always-on-top meter; click-through so the game stays playable |
| **Raid / heals / misc** | `/who all raid` roster, healing done + received, loot & chat |
| **Convenience** | Remembers last log + window positions; optional ability-name file |

---

## Everyday controls

| Menu | Use it for |
|------|------------|
| **Monitor** | Find / choose / resume / stop the character log |
| **Overlay** | Open overlay; toggle click-through |
| **Combat** | Copy parse, end fight, clear history, skip tiny fights |
| **Extras** | Ability names file; load sample fight |

### Overlay hotkeys

| Shortcut | Action |
|----------|--------|
| `Ctrl/Cmd+Shift+U` | Overlay clickable |
| `Ctrl/Cmd+Shift+L` | Click-through to game |

Run EQ **windowed** or **borderless** so the overlay can sit on top.

---

## Contributors

Feature ideas and questions: use [Discussions](https://github.com/kpxcoolx/eql-meter/discussions) (Ideas / Q&A). Bugs: open an [Issue](https://github.com/kpxcoolx/eql-meter/issues).

```bash
npm install
npm run tauri:dev
```

| Platform | Notes |
|----------|--------|
| macOS | No packaged app — [Mac + Parallels](docs/mac-parallels.md) |
| Windows | [Windows guide](docs/windows.md) · find the Legends log |

### Ship a Windows installer

Do not ask players to build. From GitHub: **Actions → windows-build → Run workflow** (e.g. `v0.1.3`), then **publish the draft release**.

Or tag: `git tag v0.1.3 && git push origin v0.1.3`

Local NSIS (on Windows):

```bash
npm run tauri:build:windows
```

Output: `src-tauri/target/release/bundle/nsis/`

Parser fixtures for offline tests: [`samples/`](samples/).
