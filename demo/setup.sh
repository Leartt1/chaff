#!/usr/bin/env bash
# Builds a throwaway demo workspace + fake caches for the chaff demo gif (macOS).
# Uses `mkfile -n` (sparse) so the big sizes cost ~no real disk.
set -e

P=/tmp/demo-projects
H=/tmp/demo-home
E=/tmp/demo-empty
rm -rf "$P" "$H" "$E"
mkdir -p "$E"

# --- fake projects ---
mkdir -p "$P/web-app/node_modules" "$P/web-app/src" \
         "$P/api/target" "$P/ml/.venv" "$P/site/.next"
echo '{}' > "$P/web-app/package.json"
: > "$P/api/Cargo.toml"
echo '{}' > "$P/site/package.json"
mkfile -n 420m  "$P/web-app/node_modules/deps.bin"
mkfile -n 1200m "$P/api/target/build.bin"
mkfile -n 380m  "$P/ml/.venv/site-packages.bin"
mkfile -n 210m  "$P/site/.next/cache.bin"
touch -t 202604010000 "$P/api/target"
touch -t 202601150000 "$P/ml/.venv"

# --- fake global caches (chaff reads $HOME, so the tape sets HOME=$H) ---
mkdir -p "$H/.npm/_cacache" "$H/Library/pnpm/store" \
         "$H/Library/Caches/pip" "$H/.cargo/registry/cache" \
         "$H/.cache/huggingface"
mkfile -n 6800m "$H/.npm/_cacache/index.bin"
mkfile -n 930m  "$H/Library/pnpm/store/store.bin"
mkfile -n 718m  "$H/Library/Caches/pip/wheels.bin"
mkfile -n 1200m "$H/.cargo/registry/cache/crates.bin"
mkfile -n 148m  "$H/.cache/huggingface/models.bin"
