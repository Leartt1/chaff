# chaff

Safe, smart dev-disk reclaimer — winnow the chaff from your projects.

> Status: v0.6

[![CI](https://github.com/Leartt1/chaff/actions/workflows/ci.yml/badge.svg)](https://github.com/Leartt1/chaff/actions/workflows/ci.yml) [![crates.io](https://img.shields.io/crates/v/chaff.svg)](https://crates.io/crates/chaff) [![downloads](https://img.shields.io/crates/d/chaff.svg)](https://crates.io/crates/chaff)

![chaff in action](demo/demo.gif)

Dev machines slowly fill with **regenerable** junk: `node_modules`, `target`,
`.venv`, `__pycache__`, `dist`, `.next`, and bloated package-manager caches.
It's all re-creatable, yet it quietly costs tens of gigabytes.

`chaff` finds that space and reclaims it — without the fear of deleting
something you actually needed.

## Safe by design

The one rule: **never remove anything you can't get back.**

- Deletes to the system **trash**, not `rm -rf` — recoverable if you change your mind.
- Only touches known **regenerable** artifacts; never your source.
- Never deletes **git-tracked** files — only true throwaway artifacts (`--force` to override).
- **Dry-run by default** — `chaff clean` previews; nothing goes until you add `--apply`.
- **Protect anything** with a `.chaffignore` (glob patterns) or a config file.

## Smart

- Sort and filter by **size**, **age**, and **type**.
- One pass across all your projects, every ecosystem at once.
- Covers node, rust, python, next, nuxt, svelte, gradle, dart, terraform, cocoapods, swift, elixir, haskell, zig, and .NET — plus tool caches (pytest, mypy, ruff, tox, turbo, parcel, angular, astro, docusaurus, nyc, coverage).
- Optionally sweep global caches too (`--caches`): npm, pnpm, yarn, pip, uv, cargo, go, gradle, maven, Hugging Face, Xcode DerivedData, Homebrew, deno, composer.

## Usage

```sh
chaff scan                 # show reclaimable space, biggest first
chaff scan ~/code ~/work   # scan specific roots
chaff scan --caches        # include global package-manager caches
chaff scan --json | jq     # machine-readable output for scripts/CI
chaff scan --min-size 100M # only items at least 100 MB
chaff scan --top 10        # only the 10 largest (total still counts everything)

chaff clean                # interactive picker (↑/↓ · space · a all · / search · s sort · enter)
chaff clean --older-than 30d --type node   # targeted
chaff clean --all --apply  # reclaim everything safe, for real (to trash)
chaff clean --json         # JSON of what clean would reclaim (never deletes)
chaff clean --all --apply --purge   # permanently delete (frees space now; not recoverable)

chaff completions zsh      # shell completions (bash/zsh/fish/elvish/powershell)
```

## Configuration

Set defaults in `~/.config/chaff/config.toml` (or point `$CHAFF_CONFIG` at a file):

```toml
older_than = "30d"        # default age filter for `clean`
caches = true             # include global caches by default
ignore = ["**/vendor/**"] # always-protected globs
```

Protect paths with a `.chaffignore` — in any scanned root, or globally at
`~/.config/chaff/.chaffignore`. Glob patterns; a bare name protects any
directory with that name:

```
keepme            # protects every keepme/ dir and its contents
build/            # trailing slash works (gitignore-style)
app/dist          # sub-paths match at any depth
**/fixtures/**    # explicit globs work too
```

CLI flags always override config (e.g. `--no-caches` turns off the config's
`caches` for a single run). An invalid ignore pattern is reported, never
silently dropped.

## Roadmap

- Scheduled / automatic reclaim
- Custom artifact rules via config

## GitHub Action

Free disk on a CI runner — fixes "no space left on device", and stops self-hosted
runners filling up between jobs:

```yaml
- uses: Leartt1/chaff@v0.6.1
  with:
    caches: true        # also clear package-manager caches
    # paths: .          # roots to scan (default: .)
    # older_than: 30d
    # min_size: 100M
    # dry_run: true     # preview without deleting
```

On CI it **permanently** deletes (via `--purge`) so space is actually freed —
git-tracked files are still never touched. Downloads a prebuilt binary
(Linux/macOS runners), falling back to `cargo install`.

## Install

```sh
cargo install chaff
```

## Alternatives

If `chaff` isn't your style, these solve nearby problems and are worth a look:
[kondo](https://github.com/tbillington/kondo), [npkill](https://github.com/voidcosmos/npkill),
and [cargo-sweep](https://github.com/holmgr/cargo-sweep).

## License

MIT

---

> The name: _winnowing_ separates **chaff** (worthless husks — your regenerable
> junk) from **grain** (your real code, left untouched).
