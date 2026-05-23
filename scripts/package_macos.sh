#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist/macos/RustSurvivors"

cd "$ROOT_DIR"

cargo build -p game --bin survivor --release

rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

cp "$ROOT_DIR/target/release/survivor" "$DIST_DIR/"
cp -R "$ROOT_DIR/assets" "$DIST_DIR/"

find "$DIST_DIR" \( -name '._*' -o -name '.DS_Store' \) -delete

test -x "$DIST_DIR/survivor"
test -d "$DIST_DIR/assets"

echo "Packaged macOS folder: $DIST_DIR"
