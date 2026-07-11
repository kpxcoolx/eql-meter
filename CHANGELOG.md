# Changelog

All notable changes to EQL Meter are documented here.

Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versioning follows the GitHub release tags (`v0.1.x`). Releases currently ship a **Windows** NSIS installer only; there is no macOS `.dmg`.

## [Unreleased]

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

[Unreleased]: https://github.com/kpxcoolx/eql-meter/compare/v0.1.9...HEAD
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
