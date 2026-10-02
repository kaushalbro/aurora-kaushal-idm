#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
EXT_DIR="$ROOT_DIR/apps/aurora-extension"
DIST_DIR="$ROOT_DIR/dist"
VERSION="0.1.0"
ZIP_NAME="aurora-extension-v${VERSION}.zip"

echo "============================================================"
echo " ⚡ AURORA Kaushal IDM - WebAssembly Extension Builder"
echo "============================================================"

# Step 1: Generate crisp pixel-perfect icons
echo "[1/4] Generating extension icons..."
python3 "$SCRIPT_DIR/generate_icons.py"

# Step 2: Compile Rust WASM crate
echo "[2/4] Compiling Rust core to WebAssembly (wasm32-unknown-unknown)..."
cargo build --package aurora-wasm --target wasm32-unknown-unknown --release

# Step 3: Run wasm-bindgen to generate JS bindings
echo "[3/4] Generating WebAssembly JavaScript bindings via wasm-bindgen..."
mkdir -p "$EXT_DIR/pkg"
wasm-bindgen "$ROOT_DIR/target/wasm32-unknown-unknown/release/aurora_wasm.wasm" \
  --target web \
  --out-dir "$EXT_DIR/pkg"

# Step 4: Package clean ZIP for Chrome Web Store / distribution
echo "[4/4] Packaging extension for distribution..."
mkdir -p "$DIST_DIR"
cd "$EXT_DIR"

rm -f "$DIST_DIR/$ZIP_NAME"
if command -v zip >/dev/null 2>&1; then
  zip -r "$DIST_DIR/$ZIP_NAME" \
    manifest.json \
    background/ \
    content/ \
    core/ \
    popup/ \
    options/ \
    icons/ \
    pkg/ \
    -x "*.DS_Store" -x "*__pycache__*"
else
  python3 -c "
import shutil, os
import zipfile

dist_zip = '$DIST_DIR/$ZIP_NAME'
with zipfile.ZipFile(dist_zip, 'w', zipfile.ZIP_DEFLATED) as z:
    for folder in ['background', 'content', 'core', 'popup', 'options', 'icons', 'pkg']:
        for root, _, files in os.walk(folder):
            for file in files:
                if not file.startswith('.'):
                    filepath = os.path.join(root, file)
                    z.write(filepath)
    z.write('manifest.json')
print('Created zip via python zipfile')
"
fi

echo ""
echo "============================================================"
echo " ✅ Extension Build Complete!"
echo "============================================================"
echo ""
echo "📁 Unpacked Extension Folder (for testing):"
echo "   $EXT_DIR"
echo ""
echo "📦 Packed Chrome Web Store ZIP:"
echo "   $DIST_DIR/$ZIP_NAME"
echo ""
echo "🚀 How to Load in Chrome / Brave / Edge:"
echo "   1. Open 'chrome://extensions' in your browser."
echo "   2. Enable 'Developer mode' (top right toggle)."
echo "   3. Click 'Load unpacked' and select the folder:"
echo "      $EXT_DIR"
echo "============================================================"
