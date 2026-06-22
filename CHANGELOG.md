# Changelog

All notable changes to chaff are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com/); versions follow
[SemVer](https://semver.org/) (pre-1.0: minor = features, patch = fixes).

## [0.6.1] - 2026-06-22

### Added
- Release workflow now ships prebuilt binaries (Linux/macOS/Windows) on each
  tagged release — the first tag that activates the GitHub Action's fast install
  path (it otherwise falls back to `cargo install`).

## [0.6.0] - 2026-06-22

### Added
- **GitHub Action** (`uses: Leartt1/chaff@v0.6.0`) to reclaim disk on CI and
  self-hosted runners, plus a release workflow that ships prebuilt binaries.
- `--purge` to permanently delete instead of trashing (frees space immediately;
  not recoverable). Trash remains the default.
- Interactive picker upgrades: cycle sort with `s` (size / age / name), filter
  with `/`, and `a` now selects all *visible* items (filter, then select a type).
- `clean --json` — machine-readable preview of what would be reclaimed (never deletes).
- `--min-size` filter for `scan` and `clean` (e.g. `--min-size 100M`).
- Shell completions: `chaff completions <bash|zsh|fish|elvish|powershell>`.
- Detect more tool caches: `.pytest_cache`, `.mypy_cache`, `.ruff_cache`,
  `.tox`, `.turbo`, `.parcel-cache`.

## [0.5.0] - 2026-06-18

### Added
- `scan --json` for scripting and CI.

## [0.4.0] - 2026-06-18

### Added
- `config.toml` (default `older_than`, `caches`, `ignore`) and `.chaffignore`
  glob protection (global + per-root).

### Fixed
- Hardened ignore-glob matching: wildcard names, trailing slashes, sub-paths,
  and `**/x/**` now also protect the directory itself; invalid patterns warn.

## [0.3.0] - 2026-06-17

### Added
- Broader coverage: terraform, cocoapods, swift, elixir, haskell, zig, and .NET
  (`bin`/`obj`, gated by a project file). New caches: Xcode DerivedData,
  Homebrew, deno, composer.

## [0.2.0] - 2026-06-17

### Added
- `--caches` to reclaim global package-manager caches (npm, pnpm, yarn, pip, uv,
  cargo, go, gradle, maven, Hugging Face).

## [0.1.0] - 2026-06-17

### Added
- Initial release: `scan` and `clean` with an interactive TUI picker, size/age
  reporting, dry-run by default, recoverable delete (to trash), and protection of
  git-tracked files.

[0.6.1]: https://github.com/Leartt1/chaff/releases/tag/v0.6.1
[0.6.0]: https://github.com/Leartt1/chaff/releases/tag/v0.6.0
[0.5.0]: https://github.com/Leartt1/chaff/releases/tag/v0.5.0
[0.4.0]: https://github.com/Leartt1/chaff/releases/tag/v0.4.0
[0.3.0]: https://github.com/Leartt1/chaff/releases/tag/v0.3.0
[0.2.0]: https://github.com/Leartt1/chaff/releases/tag/v0.2.0
[0.1.0]: https://github.com/Leartt1/chaff/releases/tag/v0.1.0
