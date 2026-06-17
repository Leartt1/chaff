# chaff

Safe, smart dev-disk reclaimer — winnow the chaff from your projects.

> Status: v0.1, in active development.

[![CI](https://github.com/Leartt1/chaff/actions/workflows/ci.yml/badge.svg)](https://github.com/Leartt1/chaff/actions/workflows/ci.yml)

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

## Smart

- Sort and filter by **size**, **age**, and **type**.
- One pass across all your projects, every ecosystem at once.
- Covers node, rust, python (uv/pip/venv), next, nuxt, svelte, gradle, and dart.

## Usage

```sh
chaff scan                 # show reclaimable space, biggest first
chaff scan ~/code ~/work   # scan specific roots

chaff clean                # interactive picker — choose what to reclaim
chaff clean --older-than 30d --type node   # targeted
chaff clean --all --apply  # reclaim everything safe, for real (to trash)
```

## Roadmap

- Global package-manager caches: pnpm, pip, cargo, Hugging Face

## Install

```sh
cargo install chaff        # on first release
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
