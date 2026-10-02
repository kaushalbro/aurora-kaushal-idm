#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
EXT_DIR="$ROOT_DIR/apps/aurora-extension"
FIREFOX_DIR="$ROOT_DIR/apps/aurora-firefox-extension"
DIST_DIR="$ROOT_DIR/dist"
VERSION="0.2.0"

echo "============================================================"
echo " ⚡ AURORA Kaushal IDM - Dual Browser Extension Builder"
echo "    Targeting: Chromium (Chrome, Brave, Edge) & Mozilla Firefox"
echo "============================================================"

# Step 1: Generate crisp pixel-perfect icons
echo "[1/5] Generating extension icons..."
python3 "$SCRIPT_DIR/generate_icons.py"

# Step 2: Compile Rust WASM crate with independent target directory
echo "[2/5] Compiling Rust core to WebAssembly (wasm32-unknown-unknown)..."
cargo build --package aurora-wasm --target wasm32-unknown-unknown --release --target-dir "$ROOT_DIR/target/wasm-target"

# Step 3: Run wasm-bindgen to generate JS bindings
echo "[3/5] Generating WebAssembly JavaScript bindings via wasm-bindgen..."
mkdir -p "$EXT_DIR/pkg"
wasm-bindgen "$ROOT_DIR/target/wasm-target/wasm32-unknown-unknown/release/aurora_wasm.wasm" \
  --target web \
  --out-dir "$EXT_DIR/pkg"

# Step 4: Prepare Firefox Extension directory & manifest
echo "[4/5] Preparing Firefox WebExtension package..."
mkdir -p "$FIREFOX_DIR"
rm -rf "$FIREFOX_DIR"/*
cp -r "$EXT_DIR/background" "$FIREFOX_DIR/"
cp -r "$EXT_DIR/content" "$FIREFOX_DIR/"
cp -r "$EXT_DIR/core" "$FIREFOX_DIR/"
cp -r "$EXT_DIR/popup" "$FIREFOX_DIR/"
cp -r "$EXT_DIR/options" "$FIREFOX_DIR/"
cp -r "$EXT_DIR/offscreen" "$FIREFOX_DIR/"
cp -r "$EXT_DIR/icons" "$FIREFOX_DIR/"
cp -r "$EXT_DIR/pkg" "$FIREFOX_DIR/"

# Create Firefox-optimized manifest.json
cat > "$FIREFOX_DIR/manifest.json" << 'EOF'
{
  "manifest_version": 3,
  "name": "Aurora Kaushal Download Manager - Nepal",
  "version": "0.2.0",
  "description": "High-performance adaptive multi-connection download manager powered by pure WebAssembly and ECT scheduling.",
  "icons": {
    "16": "icons/icon-16.png",
    "32": "icons/icon-32.png",
    "48": "icons/icon-48.png",
    "128": "icons/icon-128.png"
  },
  "action": {
    "default_popup": "popup/popup.html",
    "default_title": "Aurora Kaushal Download Manager - Nepal",
    "default_icon": {
      "16": "icons/icon-16.png",
      "32": "icons/icon-32.png",
      "48": "icons/icon-48.png",
      "128": "icons/icon-128.png"
    }
  },
  "background": {
    "scripts": ["background/service-worker.js"],
    "type": "module"
  },
  "browser_specific_settings": {
    "gecko": {
      "id": "aurora-kaushal-idm@nepal.org",
      "strict_min_version": "109.0"
    }
  },
  "content_security_policy": {
    "extension_pages": "script-src 'self' 'wasm-unsafe-eval'; object-src 'self'"
  },
  "options_ui": {
    "page": "options/options.html",
    "open_in_tab": false
  },
  "content_scripts": [
    {
      "matches": ["<all_urls>"],
      "js": ["content/interceptor.js"],
      "css": ["content/interceptor.css"],
      "run_at": "document_start",
      "all_frames": true
    }
  ],
  "permissions": [
    "downloads",
    "storage",
    "contextMenus",
    "alarms",
    "notifications"
  ],
  "host_permissions": [
    "<all_urls>"
  ],
  "web_accessible_resources": [
    {
      "resources": [
        "pkg/*"
      ],
      "matches": [
        "<all_urls>"
      ]
    }
  ]
}
EOF

# Step 5: Prepare Safari WebExtension package
SAFARI_DIR="$ROOT_DIR/apps/aurora-safari-extension"
echo "[5/6] Preparing Apple Safari WebExtension package..."
mkdir -p "$SAFARI_DIR"
rm -rf "$SAFARI_DIR"/*
cp -r "$EXT_DIR/background" "$SAFARI_DIR/"
cp -r "$EXT_DIR/content" "$SAFARI_DIR/"
cp -r "$EXT_DIR/core" "$SAFARI_DIR/"
cp -r "$EXT_DIR/popup" "$SAFARI_DIR/"
cp -r "$EXT_DIR/options" "$SAFARI_DIR/"
cp -r "$EXT_DIR/offscreen" "$SAFARI_DIR/"
cp -r "$EXT_DIR/icons" "$SAFARI_DIR/"
cp -r "$EXT_DIR/pkg" "$SAFARI_DIR/"

cat > "$SAFARI_DIR/manifest.json" << 'EOF'
{
  "manifest_version": 3,
  "name": "Aurora Kaushal Download Manager - Nepal",
  "version": "0.2.0",
  "description": "High-performance adaptive multi-connection download manager powered by pure WebAssembly and ECT scheduling.",
  "icons": {
    "16": "icons/icon-16.png",
    "32": "icons/icon-32.png",
    "48": "icons/icon-48.png",
    "128": "icons/icon-128.png"
  },
  "action": {
    "default_popup": "popup/popup.html",
    "default_title": "Aurora Kaushal Download Manager - Nepal",
    "default_icon": {
      "16": "icons/icon-16.png",
      "32": "icons/icon-32.png",
      "48": "icons/icon-48.png",
      "128": "icons/icon-128.png"
    }
  },
  "background": {
    "service_worker": "background/service-worker.js",
    "type": "module"
  },
  "content_security_policy": {
    "extension_pages": "script-src 'self' 'wasm-unsafe-eval'; object-src 'self'"
  },
  "options_ui": {
    "page": "options/options.html",
    "open_in_tab": false
  },
  "content_scripts": [
    {
      "matches": ["<all_urls>"],
      "js": ["content/interceptor.js"],
      "css": ["content/interceptor.css"],
      "run_at": "document_start",
      "all_frames": true
    }
  ],
  "permissions": [
    "downloads",
    "storage",
    "contextMenus",
    "alarms",
    "notifications"
  ],
  "host_permissions": [
    "<all_urls>"
  ],
  "web_accessible_resources": [
    {
      "resources": [
        "pkg/*"
      ],
      "matches": [
        "<all_urls>"
      ]
    }
  ]
}
EOF

# Step 6: Mirror dedicated browser folders for Chrome, Brave, and Edge
CHROME_DIR="$ROOT_DIR/apps/aurora-chrome-extension"
BRAVE_DIR="$ROOT_DIR/apps/aurora-brave-extension"
EDGE_DIR="$ROOT_DIR/apps/aurora-edge-extension"

echo "[6/7] Synchronizing named browser folders for Chrome, Brave, and Edge..."
for BROWSER_DIR in "$CHROME_DIR" "$BRAVE_DIR" "$EDGE_DIR"; do
  mkdir -p "$BROWSER_DIR"
  rm -rf "$BROWSER_DIR"/*
  cp -r "$EXT_DIR"/* "$BROWSER_DIR/"
done

# Step 7: Package ZIPs & XPI for distribution
echo "[7/7] Packaging all browser distribution artifacts..."
mkdir -p "$DIST_DIR"

# Package Chrome / Brave / Edge ZIPs
CHROME_ZIP="aurora-chrome-v${VERSION}.zip"
BRAVE_ZIP="aurora-brave-v${VERSION}.zip"
EDGE_ZIP="aurora-edge-v${VERSION}.zip"
LEGACY_ZIP="aurora-extension-v${VERSION}.zip"

(
  cd "$EXT_DIR"
  zip -r "$DIST_DIR/$CHROME_ZIP" . -x "*.DS_Store" -x "*__pycache__*"
  cp "$DIST_DIR/$CHROME_ZIP" "$DIST_DIR/$BRAVE_ZIP"
  cp "$DIST_DIR/$CHROME_ZIP" "$DIST_DIR/$EDGE_ZIP"
  cp "$DIST_DIR/$CHROME_ZIP" "$DIST_DIR/$LEGACY_ZIP"
)

# Package Firefox ZIP & XPI
FIREFOX_ZIP="aurora-firefox-v${VERSION}.zip"
FIREFOX_XPI="aurora-firefox-v${VERSION}.xpi"
(
  cd "$FIREFOX_DIR"
  zip -r "$DIST_DIR/$FIREFOX_ZIP" . -x "*.DS_Store" -x "*__pycache__*"
  cp "$DIST_DIR/$FIREFOX_ZIP" "$DIST_DIR/$FIREFOX_XPI"
)

# Package Safari ZIP
SAFARI_ZIP="aurora-safari-v${VERSION}.zip"
(
  cd "$SAFARI_DIR"
  zip -r "$DIST_DIR/$SAFARI_ZIP" . -x "*.DS_Store" -x "*__pycache__*"
)

echo ""
echo "============================================================"
echo " 🌍 Symmetrical Named Browser Folders & Packages Complete!"
echo "============================================================"
echo ""
echo "🌐 GOOGLE CHROME:"
echo "   📁 Folder: $CHROME_DIR"
echo "   📦 ZIP:    $DIST_DIR/$CHROME_ZIP"
echo "   👉 Load:   'chrome://extensions' -> 'Load unpacked'"
echo ""
echo "🦁 BRAVE BROWSER:"
echo "   📁 Folder: $BRAVE_DIR"
echo "   📦 ZIP:    $DIST_DIR/$BRAVE_ZIP"
echo "   👉 Load:   'brave://extensions' -> 'Load unpacked'"
echo ""
echo "🌊 MICROSOFT EDGE:"
echo "   📁 Folder: $EDGE_DIR"
echo "   📦 ZIP:    $DIST_DIR/$EDGE_ZIP"
echo "   👉 Load:   'edge://extensions' -> 'Load unpacked'"
echo ""
echo "🦊 MOZILLA FIREFOX:"
echo "   📁 Folder: $FIREFOX_DIR"
echo "   📦 Addon:  $DIST_DIR/$FIREFOX_XPI (ZIP: $DIST_DIR/$FIREFOX_ZIP)"
echo "   👉 Load:   'about:debugging#/runtime/this-firefox' -> 'Load Temporary Add-on...'"
echo ""
echo "🍏 APPLE SAFARI:"
echo "   📁 Folder: $SAFARI_DIR"
echo "   📦 ZIP:    $DIST_DIR/$SAFARI_ZIP"
echo "   👉 Load:   Safari Develop Menu or xcrun safari-web-extension-converter"
echo "============================================================"
