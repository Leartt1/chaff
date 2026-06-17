# chaff — design

**Safe, smart dev-disk reclaimer.** *Winnow the chaff from your projects.*

## Problem

Dev machines fill up with **regenerable** junk: `node_modules`, `target`, `.venv`,
`__pycache__`, `dist`, `.next`, build dirs — plus global package-manager caches
(pnpm store, pip, cargo registry, Hugging Face models, docker). It is all
re-creatable, but it silently eats tens of GB.

Existing tools tend to be single-ecosystem (npkill = node, cargo-sweep = rust)
or general-purpose cleaners with no safety net beyond a confirmation prompt — no
recoverable delete, no size sorting, no global caches, no awareness of
uncommitted work. That's the gap chaff fills.

## Positioning (the wedge)

`chaff` is the one you **trust**:

- **Safe by design** — deletes to the OS **trash** (recoverable), never touches
  git-**tracked** files, skips projects with **uncommitted/unpushed** work unless
  forced, honors `.chaffignore`.
- **Smart** — git + age heuristics ("pushed and untouched for 60 days → safe to
  reclaim"), sort & filter by **size**, age, and type.
- **Complete** — per-project artifacts **and** global caches; current ecosystem
  coverage (node/pnpm, python/uv, rust, go, java, .next, …).

Metaphor: *winnowing* separates **chaff** (worthless husks = regenerable junk)
from **grain** (your real code = never touched).

## v1 scope

```
chaff scan [PATHS...]        # discover reclaimable space
chaff clean [PATHS...]       # reclaim it (interactive TUI or flags)
```

`scan` → a table sorted by size: `path · type · size · age (last used) · git-state`.

`clean` →
- interactive **ratatui** TUI with multi-select, **or**
- non-interactive flags: `--type node,python`, `--older-than 30d`, `--all`,
  `--include-caches`.
- **dry-run by default**; `--apply` performs the reclaim (to trash).
- prints a `reclaimed N.N GB` report.

Safety guards (always on unless `--force`): skip git-tracked paths, skip
projects with a dirty or unpushed working tree, never follow symlinks out of the
scan root, refuse to operate on `$HOME` root / system dirs.

## Architecture (isolated units)

| Module | Responsibility |
|--------|----------------|
| `cli`     | clap arg parsing → typed `Command` |
| `model`   | `Reclaimable { path, kind, size, last_used, git_state }`, enums |
| `rules`   | ecosystem definitions: which dirs are regenerable for which project marker |
| `scan`    | walk filesystem (`ignore`), match rules, collect `Reclaimable`s |
| `caches`  | locate global package-manager caches |
| `size`    | parallel recursive directory size (`rayon`) |
| `gitinfo` | per-path git state (tracked? dirty? pushed? last-commit age) |
| `clean`   | safety checks + delete-to-trash + dry-run |
| `report`  | human-readable formatting |
| `tui`     | interactive selection (ratatui) |

Each unit is independently testable; the scan/rules/size/clean core is pure and
unit-tested without touching a real terminal.

## Non-goals (v1)

No daemon/scheduling, no cloud, no GUI, no auto-restore/regenerate. Maybe later.
