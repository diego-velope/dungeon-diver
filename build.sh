#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
DIST_DIR="$PROJECT_DIR/dist"
WEB_DIR="$PROJECT_DIR/web"

echo "==> Building WASM (release)..."
cargo build --release --target wasm32-unknown-unknown

echo "==> Packaging from web/ into dist/..."
mkdir -p "$DIST_DIR"

cp "$PROJECT_DIR/target/wasm32-unknown-unknown/release/dungeon-diver.wasm" "$DIST_DIR/dungeon-diver.wasm"
cp "$WEB_DIR"/*.html "$DIST_DIR/"

# Optional custom JS files in web/
cp "$WEB_DIR"/*.js "$DIST_DIR/" 2>/dev/null || true

# Modular TV PAL (web/pal)
if [ -d "$WEB_DIR/pal" ]; then
  cp -r "$WEB_DIR/pal" "$DIST_DIR/"
fi

# Copy assets required by web build
cp -r "$PROJECT_DIR/assets" "$DIST_DIR/"

# Download mq_js_bundle.js if missing
if [ ! -f "$DIST_DIR/mq_js_bundle.js" ]; then
  echo "==> Fetching mq_js_bundle.js..."
  curl -fsSL https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js -o "$DIST_DIR/mq_js_bundle.js"
fi

echo "✓ Done → dist/"
