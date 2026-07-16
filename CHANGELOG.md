# Changelog

All notable changes to EQL Meter are documented here.

Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versioning follows the GitHub release tags (`v0.1.x`). Releases currently ship a **Windows** NSIS installer only; there is no macOS `.dmg`.

## [Unreleased]

## [0.1.23] — 2026-07-16

### Fixed

- Only one EQL Meter window can run at a time; a second launch focuses the existing window instead of opening another copy.

## [0.1.22] — 2026-07-16

### Changed

- App icon is now the bars + sword mark (matches eqlmeter.com).

## [0.1.21] — 2026-07-16

### Fixed

- Opening EQL Meter and EQL Alerts together no longer makes the second app flash and exit. Global overlay hotkey registration is non-fatal if another app already owns the keys.

## [0.1.20] — 2026-07-13

### Changed

- Overlay ranks pets on their own rows (e.g. `Kenkyo's warder`) instead of only folding pet damage into the owner total.

## [0.1.19] — 2026-07-13

### Fixed

- Beastlord warder DPS was dropped entirely: live logs use `Kenkyo\`s warder` (and similar), which was treated as an NPC instead of your pet. Possessive pet labels (`…'s` / `…\`s` warder, pet, familiar, elemental) now merge onto the owner in the overlay and meter.

### Changed

- Main window shows **Unlock Overlay** / **Lock Overlay** when the overlay is open, so you can move it again after locking (locked mode passes clicks through to the game, so the overlay itself can't be clicked).

## [0.1.18] — 2026-07-12

### Fixed

- Overlay phantoms: `itself` and `Name over time` no longer appear as separate combatants (HoT / self-heal labels).
- Named pets that hit before they were recognized are folded back into the owner (no more separate pet DPS rows once the pet is learned).

### Changed

- Breakdown rolls pet abilities into one row per pet (`Pet (Pickle)`, …) instead of listing every pet ability separately.

## [0.1.17] — 2026-07-12

### Fixed

- Windows overlay black unmovable box: create the overlay WebView on startup (async-safe), hide instead of destroy, strip `crossorigin` on overlay assets, and ship a visible HTML boot shell with drag even if React fails to mount.

## [0.1.16] — 2026-07-12

### Changed

- Overlay uses a clear **setup → lock** flow on Windows: opens clickable so you can drag it, then **Lock** sends clicks to the game (same idea as EQLogParser’s configure vs live meter).

### Fixed

- Windows overlay drag-to-move: whole meter is draggable in setup via `startDragging` (not a tiny HTML gap WebView2 often missed).

## [0.1.10] — 2026-07-11

### Fixed

- Overlay open no longer pops a blocking Windows dialog (was stealing focus / looking blank).
- Auto-detect no longer sticks disabled after the first click.
- Fights rail: Select all multi-select styling cleaned up; Clear selection vs Delete all clarified.

## [0.1.9] — 2026-07-11

### Fixed

- Windows overlay blank black box: relative asset paths + opaque WebView chrome so UI paints.
- Overlay close: main **Close Overlay** closes whenever the overlay window exists (flag drift).

## [0.1.8] — 2026-07-11

### Changed

- Relabeled **Misc** to **Loot** (loot only; chat / rolls removed from that tab).
- **Copy Parse** and **Overlay** show a native confirmation dialog (success, position, or error).

### Fixed

- Pets like Kasarn no longer appear as fights when a named mob (e.g. Innoruuk's Chosen) hits them.
- Windows overlay uses opaque chrome and always opens on a visible monitor; status reports open position.

## [0.1.7] — 2026-07-11

### Fixed

- Check for updates no longer hangs forever on "Checking…" (12s timeout + always-available Latest release link).

## [0.1.6] — 2026-07-11

### Fixed

- Fight selection (click / Select all) no longer snaps back to the newest fight after combat.
- Copy Parse works reliably out of combat for the selected fight.

## [0.1.5] — 2026-07-11

### Fixed

- Windows: overlay no longer vanishes a few seconds after opening (removed broken EQ process auto-hide).
- Overlay is forced onto a visible monitor after create.

## [0.1.4] — 2026-07-11

### Added

- In-app updates: startup check, **Menu → Check for updates**, and one-click install from GitHub Releases.

### Changed

- Slimmed the Menu to essentials (choose log, stop, click-through, updates).
- Restyled scrollbars to match the dark amber chrome.

### Fixed

- Overlay opening off-screen (stale multi-monitor / Parallels position) now relocates onto a visible display.

## [0.1.3] — 2026-07-11

### Added

- **Clear all** on the Fights rail to wipe active and recent fights.

### Fixed

- Windows: stop console windows flashing every few seconds (`tasklist` now runs hidden).
- Windows: overlay no longer shows as a white opaque box (transparent WebView background).
- Closing the main window now closes the overlay and fully exits the app.

## [0.1.2] — 2026-07-11

### Added

- Right-click a fight in the Fights list to delete it (live or recent).

### Fixed

- Named pets (e.g. Koner) no longer appear as separate fights when mobs hit the pet.
- Pet damage merges onto the character breakdown as `Pet (Name): …` abilities.

## [0.1.1] — 2026-07-11

### Fixed

- Merge pet and DoT damage onto the character with pet ability breakdown in the UI.
- Ignore self-hurt / cannibalize lines so they do not open a “Yourself” fight.

## [0.1.0] — 2026-07-11

### Added

- Initial EQL Meter release: live combat log companion for EverQuest Legends.
- Live fight tracking, multi-mob Combined view, ability breakdown, DPS chart.
- Floating overlay with click-through.
- Raid roster from `/who all raid`, Misc tab (loot / randoms / chat), Heals tab.
- Group DPS bars when a raid roster is known.
- Windows NSIS installer via GitHub Actions; remembers last log and window positions.
- Mac + Parallels path: run from source and attach to the VM-mounted log.

[Unreleased]: https://github.com/kpxcoolx/eql-meter/compare/v0.1.23...HEAD
[0.1.23]: https://github.com/kpxcoolx/eql-meter/releases/tag/v0.1.23
[0.1.22]: https://github.com/kpxcoolx/eql-meter/releases/tag/v0.1.22
[0.1.21]: https://github.com/kpxcoolx/eql-meter/releases/tag/v0.1.21
[0.1.20]: https://github.com/kpxcoolx/eql-meter/releases/tag/v0.1.20
[0.1.19]: https://github.com/kpxcoolx/eql-meter/releases/tag/v0.1.19
[0.1.18]: https://github.com/kpxcoolx/eql-meter/releases/tag/v0.1.18
[0.1.17]: https://github.com/kpxcoolx/eql-meter/releases/tag/v0.1.17
[0.1.16]: https://github.com/kpxcoolx/eql-meter/releases/tag/v0.1.16
[0.1.10]: https://github.com/kpxcoolx/eql-meter/releases/tag/v0.1.10
[0.1.9]: https://github.com/kpxcoolx/eql-meter/releases/tag/v0.1.9
[0.1.8]: https://github.com/kpxcoolx/eql-meter/releases/tag/v0.1.8
[0.1.7]: https://github.com/kpxcoolx/eql-meter/releases/tag/v0.1.7
[0.1.6]: https://github.com/kpxcoolx/eql-meter/releases/tag/v0.1.6
[0.1.5]: https://github.com/kpxcoolx/eql-meter/releases/tag/v0.1.5
[0.1.4]: https://github.com/kpxcoolx/eql-meter/releases/tag/v0.1.4
[0.1.3]: https://github.com/kpxcoolx/eql-meter/releases/tag/v0.1.3
[0.1.2]: https://github.com/kpxcoolx/eql-meter/releases/tag/v0.1.2
[0.1.1]: https://github.com/kpxcoolx/eql-meter/releases/tag/v0.1.1
[0.1.0]: https://github.com/kpxcoolx/eql-meter/releases/tag/v0.1.0
