#!/usr/bin/env bash
# ==============================================================================
# AURORA Kaushal IDM — All-In-One Unified Build Pipeline
# Builds:
#   1. Pure Rust Core Crates (8 engine crates)
#   2. High-Performance Native CLI (aurora-cli)
#   3. Lightweight Native GUI (aurora-gui)
#   4. WebAssembly Core Engine (aurora-wasm) -> Synced to all Browser Extensions
#   5. React 19 + Tailwind CSS 4 Frontend (aurora-desktop)
#   6. Tauri 2.0 Native Modern Desktop Application (aurora-desktop binary)
# ==============================================================================

set -e

# ANSI Color formatting
BOLD="\033[1m"
GREEN="\033[1;32m"
BLUE="\033[1;34m"
CYAN="\033[1;36m"
YELLOW="\033[1;33m"
RED="\033[1;31m"
RESET="\033[0m"

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_ROOT"

echo -e "${CYAN}${BOLD}"
echo "=========================================================================="
echo "    🚀 AURORA Kaushal IDM — Master All-In-One Build Pipeline             "
echo "    Accelerated Multi-Stream Engine • Nepal • React 19 • Tauri 2.0       "
echo "=========================================================================="
echo -e "${RESET}"

# ------------------------------------------------------------------------------
# STEP 1: Verify Prerequisites
# ------------------------------------------------------------------------------
echo -e "${BLUE}${BOLD}[1/5] Checking Toolchains & Prerequisites...${RESET}"

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Error: Rust (cargo) is not installed or not in PATH.${RESET}"
    exit 1
fi
echo -e "  ✅ Cargo: $(cargo --version)"

if ! command -v node &> /dev/null; then
    echo -e "${RED}❌ Error: Node.js is not installed or not in PATH.${RESET}"
    exit 1
fi
echo -e "  ✅ Node.js: $(node --version)"

if ! command -v npm &> /dev/null; then
    echo -e "${RED}❌ Error: npm is not installed or not in PATH.${RESET}"
    exit 1
fi
echo -e "  ✅ npm: $(npm --version)"

# ------------------------------------------------------------------------------
# STEP 2: Build Rust Workspace Engine & CLI / GUI Binaries
# ------------------------------------------------------------------------------
echo -e "\n${BLUE}${BOLD}[2/5] Building Rust Workspace Crates & Native Binaries (Release Mode)...${RESET}"
cargo build --release --workspace

echo -e "  ✅ Built Native CLI: ${GREEN}${PROJECT_ROOT}/target/release/aurora-cli${RESET}"
echo -e "  ✅ Built Native GUI: ${GREEN}${PROJECT_ROOT}/target/release/aurora-gui${RESET}"

# ------------------------------------------------------------------------------
# STEP 3: Build WebAssembly Engine for Browser Extensions
# ------------------------------------------------------------------------------
echo -e "\n${BLUE}${BOLD}[3/5] Building WebAssembly Acceleration Engine (aurora-wasm)...${RESET}"

if command -v wasm-pack &> /dev/null; then
    wasm-pack build --target web --release crates/aurora-wasm --out-dir "${PROJECT_ROOT}/crates/aurora-wasm/pkg"
    
    # Sync WASM pkg to all browser extension directories
    EXTENSIONS=(
        "apps/aurora-chrome-extension"
        "apps/aurora-brave-extension"
        "apps/aurora-edge-extension"
        "apps/aurora-firefox-extension"
        "apps/aurora-opera-extension"
    )

    for ext in "${EXTENSIONS[@]}"; do
        if [ -d "$ext" ]; then
            mkdir -p "$ext/pkg"
            cp -r crates/aurora-wasm/pkg/* "$ext/pkg/" 2>/dev/null || true
            echo -e "  ✅ Synced WASM artifacts to: ${ext}/pkg"
        fi
    done
else
    echo -e "  ${YELLOW}⚠️  wasm-pack not found. Using pre-bundled WASM packages for extensions.${RESET}"
fi

# ------------------------------------------------------------------------------
# STEP 4: Build React 19 + Tailwind CSS 4 Desktop Frontend
# ------------------------------------------------------------------------------
echo -e "\n${BLUE}${BOLD}[4/5] Building React 19 + Tailwind CSS 4 Desktop Frontend...${RESET}"
cd "${PROJECT_ROOT}/apps/aurora-desktop"

if [ ! -d "node_modules" ]; then
    echo -e "  📦 Installing npm dependencies..."
    npm install
fi

npm run build
echo -e "  ✅ React 19 Frontend bundle generated in: ${GREEN}apps/aurora-desktop/dist${RESET}"

cd "$PROJECT_ROOT"

# ------------------------------------------------------------------------------
# STEP 5: Build Tauri 2.0 Native Modern Desktop Application (Standalone Embedded)
# ------------------------------------------------------------------------------
echo -e "\n${BLUE}${BOLD}[5/5] Compiling Tauri 2.0 Native Desktop App Binary (with embedded frontend)...${RESET}"
cd "${PROJECT_ROOT}/apps/aurora-desktop"
npm run tauri build -- --no-bundle

echo -e "  ✅ Built Tauri Modern Desktop App: ${GREEN}${PROJECT_ROOT}/target/release/aurora-desktop${RESET}"

# Install desktop icons & .desktop entry for Linux desktop/dock integration
if [ -d "$HOME/.local/share" ]; then
    mkdir -p "$HOME/.local/share/icons/hicolor/512x512/apps" \
             "$HOME/.local/share/icons/hicolor/256x256/apps" \
             "$HOME/.local/share/icons/hicolor/128x128/apps" \
             "$HOME/.local/share/icons/hicolor/32x32/apps" \
             "$HOME/.local/share/applications"

    cp "${PROJECT_ROOT}/apps/aurora-desktop/src-tauri/icons/icon.png" "$HOME/.local/share/icons/hicolor/512x512/apps/aurora-desktop.png" 2>/dev/null || true
    cp "${PROJECT_ROOT}/apps/aurora-desktop/src-tauri/icons/128x128@2x.png" "$HOME/.local/share/icons/hicolor/256x256/apps/aurora-desktop.png" 2>/dev/null || true
    cp "${PROJECT_ROOT}/apps/aurora-desktop/src-tauri/icons/128x128.png" "$HOME/.local/share/icons/hicolor/128x128/apps/aurora-desktop.png" 2>/dev/null || true
    cp "${PROJECT_ROOT}/apps/aurora-desktop/src-tauri/icons/32x32.png" "$HOME/.local/share/icons/hicolor/32x32/apps/aurora-desktop.png" 2>/dev/null || true
    cp "${PROJECT_ROOT}/apps/aurora-desktop/src-tauri/icons/icon.png" "$HOME/.local/share/icons/aurora-desktop.png" 2>/dev/null || true

    command -v gtk-update-icon-cache &>/dev/null && gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
    command -v update-desktop-database &>/dev/null && update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
    echo -e "  ✅ Installed Desktop Application Launcher & High-Res App Icons"
fi

cd "$PROJECT_ROOT"

# ------------------------------------------------------------------------------
# BUILD COMPLETE SUMMARY
# ------------------------------------------------------------------------------
echo -e "\n${GREEN}${BOLD}==========================================================================${RESET}"
echo -e "${GREEN}${BOLD}    🎉 ALL BUILDS COMPLETED SUCCESSFULLY!                                ${RESET}"
echo -e "${GREEN}${BOLD}==========================================================================${RESET}"
echo -e "📦 ${BOLD}Distributable Artifacts:${RESET}"
echo -e "  1. 🖥️  ${BOLD}Modern Desktop App (Tauri + React 19):${RESET}"
echo -e "     👉 ${CYAN}${PROJECT_ROOT}/target/release/aurora-desktop${RESET}"
echo -e "  2. ⚡  ${BOLD}High-Performance Native CLI:${RESET}"
echo -e "     👉 ${CYAN}${PROJECT_ROOT}/target/release/aurora-cli${RESET}"
echo -e "  3. 🎨  ${BOLD}Standalone GUI (egui fallback):${RESET}"
echo -e "     👉 ${CYAN}${PROJECT_ROOT}/target/release/aurora-gui${RESET}"
echo -e "  4. 🌐  ${BOLD}Browser Extensions (Chrome, Brave, Firefox, Edge, Opera):${RESET}"
echo -e "     👉 ${CYAN}${PROJECT_ROOT}/apps/aurora-chrome-extension${RESET}"
echo -e "     👉 ${CYAN}${PROJECT_ROOT}/apps/aurora-firefox-extension${RESET}"
echo -e "     👉 ${CYAN}${PROJECT_ROOT}/apps/aurora-brave-extension${RESET}"
echo -e "=========================================================================="
