# Changelog

All notable changes to EQL Meter are documented here.

Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versioning follows the GitHub release tags (`v0.1.x`). Releases currently ship a **Windows** NSIS installer only; there is no macOS `.dmg`.

## [Unreleased]

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

[Unreleased]: https://github.com/kpxcoolx/eql-meter/compare/v0.1.5...HEAD
[0.1.5]: https://github.com/kpxcoolx/eql-meter/releases/tag/v0.1.5
[0.1.4]: https://github.com/kpxcoolx/eql-meter/releases/tag/v0.1.4
[0.1.3]: https://github.com/kpxcoolx/eql-meter/releases/tag/v0.1.3
[0.1.2]: https://github.com/kpxcoolx/eql-meter/releases/tag/v0.1.2
[0.1.1]: https://github.com/kpxcoolx/eql-meter/releases/tag/v0.1.1
[0.1.0]: https://github.com/kpxcoolx/eql-meter/releases/tag/v0.1.0
