#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
SAFARI_DIR="$ROOT_DIR/apps/aurora-safari-extension"
OUTPUT_DIR="$ROOT_DIR/dist/safari-xcode-project"

echo "============================================================"
echo " 🍏 AURORA Kaushal IDM - Safari App Extension Converter"
echo "============================================================"

# First run standard build to ensure latest WASM and JS are compiled
bash "$SCRIPT_DIR/build_extension.sh"

if command -v xcrun &> /dev/null && xcrun --find safari-web-extension-converter &> /dev/null; then
  echo "Found safari-web-extension-converter in Xcode command line tools."
  mkdir -p "$OUTPUT_DIR"
  xcrun safari-web-extension-converter "$SAFARI_DIR" \
    --project-location "$OUTPUT_DIR" \
    --app-name "Aurora Kaushal Download Manager" \
    --bundle-identifier "com.aurorakaushal.idm" \
    --swift \
    --no-open
  echo "✅ Xcode Project generated in: $OUTPUT_DIR"
else
  echo "ℹ️  'xcrun safari-web-extension-converter' is available on macOS with Xcode installed."
  echo "   You can convert this WebExtension anytime on a Mac by running:"
  echo "   xcrun safari-web-extension-converter '$SAFARI_DIR'"
fi
