#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist/macos/RustSurvivors"
APP_DIR="$ROOT_DIR/dist/macos/RustSurvivors.app"
MACOS_DIST_ROOT="$ROOT_DIR/dist/macos"
APP_CONTENTS="$APP_DIR/Contents"
APP_MACOS="$APP_CONTENTS/MacOS"
APP_RESOURCES="$APP_CONTENTS/Resources"

cd "$ROOT_DIR"

cargo build -p game --bin survivor --release

rm -rf "$DIST_DIR" "$APP_DIR"
mkdir -p "$DIST_DIR"

cp "$ROOT_DIR/target/release/survivor" "$DIST_DIR/"
cp -R "$ROOT_DIR/assets" "$DIST_DIR/"
cp "$ROOT_DIR/docs/ASSET_LICENSES.md" "$DIST_DIR/"
cp "$ROOT_DIR/docs/audio_assets.md" "$DIST_DIR/"

find "$DIST_DIR" \( -name '._*' -o -name '.DS_Store' \) -delete

test -x "$DIST_DIR/survivor"
test -d "$DIST_DIR/assets"
test -f "$DIST_DIR/ASSET_LICENSES.md"
test -f "$DIST_DIR/audio_assets.md"
(cd "$DIST_DIR" && find . -type f ! -name PACKAGE_MANIFEST.sha256 ! -name '._*' ! -name '.DS_Store' | LC_ALL=C sort | while read -r file; do shasum -a 256 "$file"; done > PACKAGE_MANIFEST.sha256)
find "$DIST_DIR" \( -name '._*' -o -name '.DS_Store' \) -delete
test -f "$DIST_DIR/PACKAGE_MANIFEST.sha256"

mkdir -p "$APP_MACOS" "$APP_RESOURCES"
cp "$ROOT_DIR/target/release/survivor" "$APP_MACOS/survivor-bin"
cp -R "$ROOT_DIR/assets" "$APP_RESOURCES/"
cp "$ROOT_DIR/docs/ASSET_LICENSES.md" "$APP_RESOURCES/"
cp "$ROOT_DIR/docs/audio_assets.md" "$APP_RESOURCES/"

cat > "$APP_MACOS/RustSurvivors" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

APP_CONTENTS="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$APP_CONTENTS/Resources"
exec "$APP_CONTENTS/MacOS/survivor-bin" "$@"
EOF
chmod +x "$APP_MACOS/RustSurvivors" "$APP_MACOS/survivor-bin"

cat > "$APP_CONTENTS/Info.plist" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>ko</string>
  <key>CFBundleExecutable</key>
  <string>RustSurvivors</string>
  <key>CFBundleIdentifier</key>
  <string>com.chunsam.rustsurvivors</string>
  <key>CFBundleName</key>
  <string>Rust Survivors</string>
  <key>CFBundleDisplayName</key>
  <string>Rust Survivors</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>0.1.0</string>
  <key>CFBundleVersion</key>
  <string>0.1.0</string>
  <key>LSMinimumSystemVersion</key>
  <string>11.0</string>
</dict>
</plist>
EOF

find "$APP_DIR" \( -name '._*' -o -name '.DS_Store' \) -delete
find "$MACOS_DIST_ROOT" -maxdepth 1 \( -name '._*' -o -name '.DS_Store' \) -delete

test -x "$APP_MACOS/RustSurvivors"
test -x "$APP_MACOS/survivor-bin"
test -d "$APP_RESOURCES/assets"
test -f "$APP_RESOURCES/ASSET_LICENSES.md"
test -f "$APP_RESOURCES/audio_assets.md"
test -f "$APP_CONTENTS/Info.plist"
(cd "$APP_DIR" && find . -type f ! -name PACKAGE_MANIFEST.sha256 ! -name '._*' ! -name '.DS_Store' | LC_ALL=C sort | while read -r file; do shasum -a 256 "$file"; done > PACKAGE_MANIFEST.sha256)
find "$APP_DIR" \( -name '._*' -o -name '.DS_Store' \) -delete
find "$MACOS_DIST_ROOT" -maxdepth 1 \( -name '._*' -o -name '.DS_Store' \) -delete
test -f "$APP_DIR/PACKAGE_MANIFEST.sha256"

"$ROOT_DIR/scripts/verify_macos_package.sh" "$DIST_DIR" "$APP_DIR"

echo "Packaged macOS folder: $DIST_DIR"
echo "Packaged macOS app: $APP_DIR"
