#!/usr/bin/env sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
EXE=${1:-"$ROOT/target/release/yekaterina"}

if [ ! -x "$EXE" ]; then
  echo "[Yekaterina demo] Release binary not found. Building with the locked dependency set..."
  (cd "$ROOT" && cargo build --locked --release)
fi

exec python3 "$ROOT/tools/demo.py" "$EXE"
