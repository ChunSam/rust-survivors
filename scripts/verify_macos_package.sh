#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${1:-$ROOT_DIR/dist/macos/RustSurvivors}"
APP_DIR="${2:-$ROOT_DIR/dist/macos/RustSurvivors.app}"

check_no_metadata() {
  local path="$1"
  local found
  found="$(find "$path" \( -name '._*' -o -name '.DS_Store' \) -print)"
  if [[ -n "$found" ]]; then
    echo "macOS metadata files remained in package:" >&2
    echo "$found" >&2
    exit 1
  fi
}

verify_manifest() {
  local path="$1"
  test -f "$path/PACKAGE_MANIFEST.sha256"
  (cd "$path" && shasum -a 256 -c PACKAGE_MANIFEST.sha256)
}

test -x "$DIST_DIR/survivor"
test -d "$DIST_DIR/assets"
test -f "$DIST_DIR/ASSET_LICENSES.md"
test -f "$DIST_DIR/audio_assets.md"
check_no_metadata "$DIST_DIR"
verify_manifest "$DIST_DIR"

test -x "$APP_DIR/Contents/MacOS/RustSurvivors"
test -x "$APP_DIR/Contents/MacOS/survivor-bin"
test -d "$APP_DIR/Contents/Resources/assets"
test -f "$APP_DIR/Contents/Resources/ASSET_LICENSES.md"
test -f "$APP_DIR/Contents/Resources/audio_assets.md"
test -f "$APP_DIR/Contents/Info.plist"
check_no_metadata "$APP_DIR"
plutil -lint "$APP_DIR/Contents/Info.plist"
verify_manifest "$APP_DIR"

echo "Verified macOS packages:"
echo "  $DIST_DIR"
echo "  $APP_DIR"
