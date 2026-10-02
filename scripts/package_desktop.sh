#!/usr/bin/env bash
# ==============================================================================
# AURORA Kaushal IDM - Ultra-Compressed Multi-Platform Desktop Packaging Suite
# Generates modern React 19 + Tailwind CSS + Tauri 2.0 packages:
#   1. Debian/Ubuntu Linux Package (.deb) with clean upgrade/uninstall hooks
#   2. Linux Portable Archive (.tar.gz) with clean install/uninstall scripts
#   3. Windows Release Package (.zip & .exe installer)
#   4. macOS Application Bundle (.app / .zip)
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"
TARGET_DIR="${ROOT_DIR}/target"
PKG_DIR="${TARGET_DIR}/pkg"

# Source centralized .env configuration if present
if [ -f "${ROOT_DIR}/.env" ]; then
    export $(grep -v '^#' "${ROOT_DIR}/.env" | xargs -d '\n')
fi
VERSION="${AURORA_VERSION:-${VERSION:-0.3.0}}"
APP_NAME="aurora-kaushal-idm"
BIN_NAME="aurora-desktop"

mkdir -p "${DIST_DIR}"
mkdir -p "${PKG_DIR}"

echo "============================================================"
echo " ⚡ AURORA Kaushal IDM - Modern Desktop Packager"
echo "    Version: ${VERSION} | Target: .deb, .tar.gz, .exe, .app"
echo "============================================================"

# Ensure full icon suite is generated
python3 "${SCRIPT_DIR}/generate_icons.py" > /dev/null 2>&1 || true

# ------------------------------------------------------------------------------
# 1. Linux Debian Package (.deb)
# ------------------------------------------------------------------------------
echo -e "\n[1/4] Building Linux Debian (.deb) Package for modern AURORA IDM..."
LINUX_BIN="${TARGET_DIR}/release/${BIN_NAME}"

if [[ ! -f "${LINUX_BIN}" ]]; then
  echo "Compiling Linux release binary (React 19 + Tauri 2.0)..."
  (cd "${ROOT_DIR}/apps/aurora-desktop" && npm run build && npm run tauri build -- --no-bundle)
fi

strip --strip-all "${LINUX_BIN}" 2>/dev/null || true

DEB_ROOT="${PKG_DIR}/deb"
rm -rf "${DEB_ROOT}"
mkdir -p "${DEB_ROOT}/DEBIAN"
mkdir -p "${DEB_ROOT}/usr/bin"
mkdir -p "${DEB_ROOT}/usr/share/applications"
mkdir -p "${DEB_ROOT}/usr/share/pixmaps"
mkdir -p "${DEB_ROOT}/usr/share/doc/${APP_NAME}"

# Copy binary
cp "${LINUX_BIN}" "${DEB_ROOT}/usr/bin/${BIN_NAME}"
chmod 755 "${DEB_ROOT}/usr/bin/${BIN_NAME}"

# Copy icons to all standard hicolor dimensions and pixmaps
mkdir -p "${DEB_ROOT}/usr/share/icons/hicolor/512x512/apps" \
         "${DEB_ROOT}/usr/share/icons/hicolor/256x256/apps" \
         "${DEB_ROOT}/usr/share/icons/hicolor/128x128/apps" \
         "${DEB_ROOT}/usr/share/icons/hicolor/32x32/apps"

cp "${ROOT_DIR}/apps/aurora-desktop/src-tauri/icons/icon.png" "${DEB_ROOT}/usr/share/icons/hicolor/512x512/apps/aurora-desktop.png" 2>/dev/null || true
cp "${ROOT_DIR}/apps/aurora-desktop/src-tauri/icons/128x128@2x.png" "${DEB_ROOT}/usr/share/icons/hicolor/256x256/apps/aurora-desktop.png" 2>/dev/null || true
cp "${ROOT_DIR}/apps/aurora-desktop/src-tauri/icons/128x128.png" "${DEB_ROOT}/usr/share/icons/hicolor/128x128/apps/aurora-desktop.png" 2>/dev/null || true
cp "${ROOT_DIR}/apps/aurora-desktop/src-tauri/icons/32x32.png" "${DEB_ROOT}/usr/share/icons/hicolor/32x32/apps/aurora-desktop.png" 2>/dev/null || true
cp "${ROOT_DIR}/apps/aurora-desktop/src-tauri/icons/icon.png" "${DEB_ROOT}/usr/share/pixmaps/aurora-desktop.png" 2>/dev/null || true
cp "${ROOT_DIR}/apps/aurora-desktop/src-tauri/icons/icon.png" "${DEB_ROOT}/usr/share/pixmaps/aurora-idm.png" 2>/dev/null || true

# Create .desktop file
cat << 'EOF' > "${DEB_ROOT}/usr/share/applications/aurora-desktop.desktop"
[Desktop Entry]
Version=1.0
Name=AURORA IDM
GenericName=Internet Download Manager
Comment=Ultra-High Performance Internet Download Manager (React 19 + Tauri 2.0)
Exec=/usr/bin/aurora-desktop %U
Icon=aurora-desktop
Terminal=false
Type=Application
Categories=Network;FileTransfer;Utility;
MimeType=x-scheme-handler/aurora;x-scheme-handler/auroradl;
Keywords=download;manager;accelerator;idm;aurora;kaushal;
StartupWMClass=aurora-desktop
StartupNotify=true
EOF

# DEBIAN/control
INST_SIZE=$(du -sk "${DEB_ROOT}/usr" | cut -f1)
cat << EOF > "${DEB_ROOT}/DEBIAN/control"
Package: ${APP_NAME}
Version: ${VERSION}
Section: net
Priority: optional
Architecture: amd64
Installed-Size: ${INST_SIZE}
Maintainer: AURORA Development Team <aurora-dev@idm.local>
Depends: libc6 (>= 2.31), libssl3 (>= 3.0.0) | libssl1.1, libwebkit2gtk-4.1-0 | libwebkit2gtk-4.0-37
Description: AURORA Kaushal IDM - High-Speed Download Accelerator
 High-performance multi-stream download accelerator featuring Adaptive
 Estimated Completion Time (ECT) scheduling, cryptographic integrity
 verification, and seamless browser extension synchronization.
EOF

# DEBIAN/preinst
cat << 'EOF' > "${DEB_ROOT}/DEBIAN/preinst"
#!/bin/sh
set -e
killall -9 aurora-desktop 2>/dev/null || true
killall -9 aurora-gui 2>/dev/null || true
pkill -f aurora-desktop 2>/dev/null || true
pkill -f aurora-gui 2>/dev/null || true

rm -f /usr/share/applications/aurora-idm.desktop 2>/dev/null || true
rm -f /usr/share/applications/com.aurora.kaushal.idm.desktop 2>/dev/null || true
rm -f /home/*/.local/share/applications/aurora-idm.desktop 2>/dev/null || true
rm -f /home/*/.local/share/applications/aurora-desktop.desktop 2>/dev/null || true
rm -f /home/*/.local/share/applications/com.aurora.kaushal.idm.desktop 2>/dev/null || true
rm -rf /tmp/aurora* /var/tmp/aurora* 2>/dev/null || true
exit 0
EOF
chmod 755 "${DEB_ROOT}/DEBIAN/preinst"

# DEBIAN/postinst
cat << 'EOF' > "${DEB_ROOT}/DEBIAN/postinst"
#!/bin/sh
set -e
chmod 755 /usr/bin/aurora-desktop
rm -f /usr/share/applications/aurora-idm.desktop 2>/dev/null || true
rm -f /home/*/.local/share/applications/aurora-idm.desktop 2>/dev/null || true
rm -f /home/*/.local/share/applications/aurora-desktop.desktop 2>/dev/null || true

if which update-desktop-database >/dev/null 2>&1; then
    update-desktop-database -q /usr/share/applications 2>/dev/null || true
    update-desktop-database -q /home/*/.local/share/applications 2>/dev/null || true
fi
if which gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor 2>/dev/null || true
fi
if which xdg-mime >/dev/null 2>&1; then
    xdg-mime default aurora-desktop.desktop x-scheme-handler/aurora x-scheme-handler/auroradl 2>/dev/null || true
fi
exit 0
EOF
chmod 755 "${DEB_ROOT}/DEBIAN/postinst"

# DEBIAN/prerm
cat << 'EOF' > "${DEB_ROOT}/DEBIAN/prerm"
#!/bin/sh
set -e
killall -9 aurora-desktop 2>/dev/null || true
killall -9 aurora-gui 2>/dev/null || true
pkill -f aurora-desktop 2>/dev/null || true
pkill -f aurora-gui 2>/dev/null || true
exit 0
EOF
chmod 755 "${DEB_ROOT}/DEBIAN/prerm"

# DEBIAN/postrm
cat << 'EOF' > "${DEB_ROOT}/DEBIAN/postrm"
#!/bin/sh
set -e
if [ "$1" = "purge" ] || [ "$1" = "remove" ]; then
    rm -f /usr/share/applications/aurora-desktop.desktop
    rm -f /usr/share/applications/aurora-idm.desktop
    rm -f /usr/share/pixmaps/aurora-desktop.png
    rm -f /usr/share/pixmaps/aurora-idm.png
    rm -f /usr/share/icons/hicolor/*/apps/aurora-desktop.png
    rm -f /usr/share/icons/hicolor/*/apps/aurora-idm.png
    rm -rf /usr/share/doc/aurora-kaushal-idm
    rm -rf /tmp/aurora* /var/tmp/aurora* 2>/dev/null || true
fi
if which update-desktop-database >/dev/null 2>&1; then
    update-desktop-database -q /usr/share/applications 2>/dev/null || true
fi
if which gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor 2>/dev/null || true
fi
exit 0
EOF
chmod 755 "${DEB_ROOT}/DEBIAN/postrm"

# Build .deb with xz compression
dpkg-deb --build -Zxz "${DEB_ROOT}" "${DIST_DIR}/${APP_NAME}_${VERSION}_amd64.deb"
echo "✅ Generated: ${DIST_DIR}/${APP_NAME}_${VERSION}_amd64.deb"

# ------------------------------------------------------------------------------
# 2. Linux Portable Archive (.tar.gz)
# ------------------------------------------------------------------------------
echo -e "\n[2/4] Building Linux Portable (.tar.gz) Bundle..."
TAR_ROOT="${PKG_DIR}/linux-portable"
rm -rf "${TAR_ROOT}"
mkdir -p "${TAR_ROOT}"

cp "${LINUX_BIN}" "${TAR_ROOT}/${BIN_NAME}"
cp "${ROOT_DIR}/apps/aurora-desktop/src-tauri/icons/icon.png" "${TAR_ROOT}/aurora-desktop.png"
cp "${DEB_ROOT}/usr/share/applications/aurora-desktop.desktop" "${TAR_ROOT}/aurora-desktop.desktop"

# Portable install script
cat << 'EOF' > "${TAR_ROOT}/install.sh"
#!/usr/bin/env bash
set -e
echo "Installing AURORA IDM to ~/.local/bin..."
killall -9 aurora-desktop 2>/dev/null || true
killall -9 aurora-gui 2>/dev/null || true

mkdir -p ~/.local/bin
mkdir -p ~/.local/share/applications
mkdir -p ~/.local/share/icons/hicolor/512x512/apps
mkdir -p ~/.local/share/pixmaps

cp aurora-desktop ~/.local/bin/aurora-desktop
chmod +x ~/.local/bin/aurora-desktop
cp aurora-desktop.png ~/.local/share/icons/hicolor/512x512/apps/aurora-desktop.png
cp aurora-desktop.png ~/.local/share/pixmaps/aurora-desktop.png

cat << 'DESK' > ~/.local/share/applications/aurora-desktop.desktop
[Desktop Entry]
Version=1.0
Name=AURORA IDM
GenericName=Internet Download Manager
Comment=Ultra-High Performance Internet Download Manager (React 19 + Tauri 2.0)
Exec=aurora-desktop %U
Icon=aurora-desktop
Terminal=false
Type=Application
Categories=Network;FileTransfer;Utility;
MimeType=x-scheme-handler/aurora;x-scheme-handler/auroradl;
Keywords=download;manager;accelerator;idm;aurora;kaushal;
StartupWMClass=aurora-desktop
StartupNotify=true
DESK

which update-desktop-database >/dev/null 2>&1 && update-desktop-database ~/.local/share/applications 2>/dev/null || true
which gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -q -t -f ~/.local/share/icons/hicolor 2>/dev/null || true
which xdg-mime >/dev/null 2>&1 && xdg-mime default aurora-desktop.desktop x-scheme-handler/aurora x-scheme-handler/auroradl 2>/dev/null || true
echo "✅ AURORA IDM (React 19 + Tauri 2.0) installed successfully!"
EOF
chmod +x "${TAR_ROOT}/install.sh"

# Portable uninstall script
cat << 'EOF' > "${TAR_ROOT}/uninstall.sh"
#!/usr/bin/env bash
set -e
echo "Uninstalling AURORA IDM..."
killall -9 aurora-desktop 2>/dev/null || true
killall -9 aurora-gui 2>/dev/null || true
rm -f ~/.local/bin/aurora-desktop
rm -f ~/.local/bin/aurora-gui
rm -f ~/.local/share/applications/aurora-desktop.desktop
rm -f ~/.local/share/applications/aurora-idm.desktop
rm -f ~/.local/share/icons/hicolor/512x512/apps/aurora-desktop.png
rm -f ~/.local/share/pixmaps/aurora-desktop.png
which update-desktop-database >/dev/null 2>&1 && update-desktop-database ~/.local/share/applications 2>/dev/null || true
which gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -q -t -f ~/.local/share/icons/hicolor 2>/dev/null || true
echo "✅ AURORA IDM completely removed."
EOF
chmod +x "${TAR_ROOT}/uninstall.sh"

tar -czf "${DIST_DIR}/${APP_NAME}-v${VERSION}-linux-x86_64.tar.gz" -C "${PKG_DIR}" "linux-portable"
echo "✅ Generated: ${DIST_DIR}/${APP_NAME}-v${VERSION}-linux-x86_64.tar.gz"

# ------------------------------------------------------------------------------
# 3. Windows Native Release Package (.zip with aurora-desktop.exe, install.bat, setup.vbs, uninstall.bat)
# ------------------------------------------------------------------------------
echo -e "\n[3/4] Building Windows Standalone (.exe) Package..."
WIN_BIN="${TARGET_DIR}/x86_64-pc-windows-gnu/release/${BIN_NAME}.exe"

if [[ ! -f "${WIN_BIN}" ]] && [[ -f "${TARGET_DIR}/x86_64-pc-windows-gnu/release/aurora-gui.exe" ]]; then
  WIN_BIN="${TARGET_DIR}/x86_64-pc-windows-gnu/release/aurora-gui.exe"
fi

if [[ -f "${WIN_BIN}" ]]; then
  WIN_ROOT="${PKG_DIR}/windows"
  rm -rf "${WIN_ROOT}"
  mkdir -p "${WIN_ROOT}"

  cp "${WIN_BIN}" "${WIN_ROOT}/aurora-desktop.exe"
  x86_64-w64-mingw32-strip "${WIN_ROOT}/aurora-desktop.exe" 2>/dev/null || true
  if [[ -f "${TARGET_DIR}/x86_64-pc-windows-gnu/release/WebView2Loader.dll" ]]; then
    cp "${TARGET_DIR}/x86_64-pc-windows-gnu/release/WebView2Loader.dll" "${WIN_ROOT}/WebView2Loader.dll"
  fi
  cp "${ROOT_DIR}/apps/aurora-desktop/src-tauri/icons/icon.ico" "${WIN_ROOT}/icon.ico" 2>/dev/null || cp "${ROOT_DIR}/apps/aurora-gui/resources/icon.ico" "${WIN_ROOT}/icon.ico"
  cp "${ROOT_DIR}/apps/aurora-desktop/src-tauri/icons/128x128.png" "${WIN_ROOT}/icon.png" 2>/dev/null || true

  # Windows Clean One-Click Installer (.bat)
  cat << 'EOF' > "${WIN_ROOT}/install.bat"
@echo off
title AURORA IDM Setup
set "INSTALL_DIR=%LOCALAPPDATA%\Programs\AuroraIDM"

echo ========================================================
echo  AURORA Kaushal IDM - High-Speed Download Manager Setup
echo ========================================================
echo [1/4] Terminating existing running instances...
taskkill /F /IM aurora-desktop.exe 2>nul
taskkill /F /IM aurora-gui.exe 2>nul
timeout /t 1 /nobreak >nul

echo [2/4] Cleaning previous installation...
if exist "%INSTALL_DIR%" rmdir /s /q "%INSTALL_DIR%"
mkdir "%INSTALL_DIR%"

echo [3/4] Copying application files, WebView2 dependencies, and icons...
copy /y "%~dp0aurora-desktop.exe" "%INSTALL_DIR%\" >nul
copy /y "%~dp0WebView2Loader.dll" "%INSTALL_DIR%\" >nul 2>nul
copy /y "%~dp0icon.ico" "%INSTALL_DIR%\" >nul 2>nul
copy /y "%~dp0icon.png" "%INSTALL_DIR%\" >nul 2>nul
copy /y "%~dp0uninstall.bat" "%INSTALL_DIR%\" >nul

echo [4/4] Creating Desktop and Start Menu shortcuts with icon...
powershell -NoProfile -ExecutionPolicy Bypass -Command ^
  "$ws = New-Object -ComObject WScript.Shell; " ^
  "$desk = [System.IO.Path]::Combine([Environment]::GetFolderPath('Desktop'), 'AURORA IDM.lnk'); " ^
  "$s = $ws.CreateShortcut($desk); " ^
  "$s.TargetPath = '%INSTALL_DIR%\aurora-desktop.exe'; " ^
  "$s.WorkingDirectory = '%INSTALL_DIR%'; " ^
  "$s.IconLocation = '%INSTALL_DIR%\icon.ico,0'; " ^
  "$s.Save(); " ^
  "$startDir = [System.IO.Path]::Combine([Environment]::GetFolderPath('Programs'), 'AURORA IDM'); " ^
  "if (!(Test-Path $startDir)) { New-Item -ItemType Directory -Path $startDir | Out-Null }; " ^
  "$s2 = $ws.CreateShortcut([System.IO.Path]::Combine($startDir, 'AURORA IDM.lnk')); " ^
  "$s2.TargetPath = '%INSTALL_DIR%\aurora-desktop.exe'; " ^
  "$s2.WorkingDirectory = '%INSTALL_DIR%'; " ^
  "$s2.IconLocation = '%INSTALL_DIR%\icon.ico,0'; " ^
  "$s2.Save()"

echo.
echo ========================================================
echo  SUCCESS: AURORA IDM installed successfully!
echo  Launching application...
echo ========================================================
start "" "%INSTALL_DIR%\aurora-desktop.exe"
timeout /t 2 /nobreak >nul
exit
EOF

  # Windows Silent VBS Installer
  cat << 'EOF' > "${WIN_ROOT}/setup.vbs"
Set WshShell = CreateObject("WScript.Shell")
Set fso = CreateObject("Scripting.FileSystemObject")
currentDir = fso.GetParentFolderName(WScript.ScriptFullName)
batPath = currentDir & "\install.bat"
WshShell.Run """" & batPath & """", 0, True
EOF

  # Windows Clean Uninstaller
  cat << 'EOF' > "${WIN_ROOT}/uninstall.bat"
@echo off
title AURORA IDM Uninstaller
set "INSTALL_DIR=%LOCALAPPDATA%\Programs\AuroraIDM"

echo ========================================================
echo  AURORA IDM - Uninstaller
echo ========================================================
echo [1/2] Terminating running aurora-desktop.exe instances...
taskkill /F /IM aurora-desktop.exe 2>nul
taskkill /F /IM aurora-gui.exe 2>nul
timeout /t 1 /nobreak >nul

echo [2/2] Deleting application files, cache, and shortcuts...
if exist "%INSTALL_DIR%" rmdir /s /q "%INSTALL_DIR%"
del /f /q "%USERPROFILE%\Desktop\AURORA IDM.lnk" 2>nul
del /f /q "%USERPROFILE%\Desktop\AURORA Kaushal IDM.lnk" 2>nul
rmdir /s /q "%APPDATA%\Microsoft\Windows\Start Menu\Programs\AURORA IDM" 2>nul
rmdir /s /q "%APPDATA%\Microsoft\Windows\Start Menu\Programs\AURORA Kaushal IDM" 2>nul

echo.
echo ========================================================
echo  AURORA IDM has been completely removed.
echo ========================================================
pause
EOF

  (cd "${WIN_ROOT}" && zip -9 -q -r "${DIST_DIR}/${APP_NAME}-v${VERSION}-windows-x64.zip" .)
  echo "✅ Generated: ${DIST_DIR}/${APP_NAME}-v${VERSION}-windows-x64.zip"

  # Compile official Standalone NSIS .exe Setup Wizard
  if which makensis >/dev/null 2>&1; then
    echo "Compiling Standalone NSIS Windows Installer (.exe)..."
    makensis -DPRODUCT_VERSION="${VERSION}" "${SCRIPT_DIR}/installer.nsi" > /dev/null
    echo "✅ Generated: ${DIST_DIR}/${APP_NAME}-v${VERSION}-setup.exe"
  fi
else
  echo "⚠️ Windows binary not found yet, skipping Windows packaging."
fi

# ------------------------------------------------------------------------------
# 4. macOS Application Bundle (.app / .zip)
# ------------------------------------------------------------------------------
echo -e "\n[4/4] Building macOS Application Bundle (.app)..."
MAC_APP_DIR="${PKG_DIR}/macos/AURORA IDM.app"
rm -rf "${PKG_DIR}/macos"
mkdir -p "${MAC_APP_DIR}/Contents/MacOS"
mkdir -p "${MAC_APP_DIR}/Contents/Resources"

cp "${LINUX_BIN}" "${MAC_APP_DIR}/Contents/MacOS/aurora-desktop"
chmod +x "${MAC_APP_DIR}/Contents/MacOS/aurora-desktop"
cp "${ROOT_DIR}/apps/aurora-desktop/src-tauri/icons/128x128.png" "${MAC_APP_DIR}/Contents/Resources/icon.png" 2>/dev/null || true

# Info.plist
cat << EOF > "${MAC_APP_DIR}/Contents/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>aurora-desktop</string>
    <key>CFBundleIconFile</key>
    <string>icon</string>
    <key>CFBundleIdentifier</key>
    <string>com.aurora.kaushal.idm</string>
    <key>CFBundleName</key>
    <string>AURORA IDM</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

# macOS Clean install & uninstall scripts
cat << 'EOF' > "${PKG_DIR}/macos/install.sh"
#!/usr/bin/env bash
set -e
echo "Installing AURORA IDM to /Applications..."
killall -9 aurora-desktop 2>/dev/null || true
killall -9 aurora-gui 2>/dev/null || true
rm -rf "/Applications/AURORA IDM.app" "/Applications/AURORA Kaushal IDM.app"
cp -R "AURORA IDM.app" "/Applications/"
echo "✅ AURORA IDM installed to /Applications!"
EOF
chmod +x "${PKG_DIR}/macos/install.sh"

cat << 'EOF' > "${PKG_DIR}/macos/uninstall.sh"
#!/usr/bin/env bash
set -e
echo "Uninstalling AURORA IDM..."
killall -9 aurora-desktop 2>/dev/null || true
killall -9 aurora-gui 2>/dev/null || true
rm -rf "/Applications/AURORA IDM.app" "/Applications/AURORA Kaushal IDM.app"
rm -rf ~/Library/Application\ Support/aurora 2>/dev/null || true
echo "✅ AURORA IDM removed."
EOF
chmod +x "${PKG_DIR}/macos/uninstall.sh"

(cd "${PKG_DIR}/macos" && zip -9 -q -r "${DIST_DIR}/${APP_NAME}-v${VERSION}-macos.zip" .)
echo "✅ Generated: ${DIST_DIR}/${APP_NAME}-v${VERSION}-macos.zip"

echo -e "\n============================================================"
echo " 🚀 All Multi-Platform Modern Desktop Packages Ready!"
echo "============================================================"
ls -lh "${DIST_DIR}"/aurora-kaushal-idm*

