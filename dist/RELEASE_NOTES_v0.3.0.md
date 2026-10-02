# ⚡ AURORA Kaushal IDM v0.3.0 — Modern Desktop & Full Browser Suite Release

Official release of **AURORA Kaushal IDM v0.3.0**, powered by **React 19 + Tailwind CSS 4 + Tauri 2.0** desktop architecture, high-speed multi-connection WebAssembly acceleration engine, system tray integration, autostart on system boot/reboot, and universal automatic browser download interception.

---

### 🚀 What's New in v0.3.0

- **🖥️ Modern React 19 + Tauri 2.0 Desktop UI**:
  - Full rewrite replacing legacy UI with a responsive, glassmorphic React 19 + Tailwind CSS 4 interface.
  - Interactive speed graphs, connection visualizers, real-time throughput metrics, category filtering, and batch controls.

- **🔄 System Startup & Reboot Autostart**:
  - Full cross-platform autostart toggle on system boot and restart (Linux `.config/autostart` XDG desktop entries & Windows registry).
  - Quick toggle directly from Desktop UI Settings and the system tray.

- **🪟 System Tray Integration**:
  - Native system tray icon with quick actions: Show/Hide UI, Enable/Disable Autostart, Interception Status, and Exit.
  - Minimize to tray on close to maintain background download acceleration.

- **🌐 Universal Browser Interception & Fallback Protocol**:
  - Deep-link custom protocol registration (`aurora://` and `auroradl://`).
  - When the browser extension is active or when downloading from any browser without extensions, requests automatically route to AURORA IDM.
  - Symmetrical extension packages for **Google Chrome**, **Brave**, **Microsoft Edge**, **Mozilla Firefox**, and **Apple Safari**.

- **⚙️ Centralized `.env` Versioning**:
  - All workspace crates, desktop packages, installers, extension manifests, and release uploaders are controlled from a single variable in `.env`.

---

### 📦 Distribution Packages

| Package | Platform | Description |
| :--- | :--- | :--- |
| `aurora-kaushal-idm_0.3.0_amd64.deb` | Ubuntu / Debian | Native .deb installer with desktop entries & URI scheme association |
| `aurora-kaushal-idm-v0.3.0-linux-x86_64.tar.gz` | Linux (Generic) | Portable standalone binary with one-click install/uninstall scripts |
| `aurora-kaushal-idm-v0.3.0-setup.exe` | Windows 10 / 11 | Official LZMA-compressed NSIS Setup Wizard with shortcuts & protocol handlers |
| `aurora-kaushal-idm-v0.3.0-windows-x64.zip` | Windows 64-bit | Standalone portable executable with `install.bat` and `uninstall.bat` |
| `aurora-kaushal-idm-v0.3.0-macos.zip` | macOS | Universal `.app` bundle with installation scripts |
| `aurora-chrome-v0.3.0.zip` | Google Chrome | Manifest V3 Extension with WebAssembly multi-connection engine |
| `aurora-brave-v0.3.0.zip` | Brave Browser | Manifest V3 Extension optimized for Brave shields |
| `aurora-edge-v0.3.0.zip` | Microsoft Edge | Manifest V3 Extension for Edge Add-ons |
| `aurora-firefox-v0.3.0.xpi` | Mozilla Firefox | Signed WebExtension Add-on with Gecko ID `aurora-kaushal-idm@nepal.org` |
| `aurora-firefox-v0.3.0.zip` | Mozilla Firefox | Unpacked Firefox WebExtension zip |
| `aurora-safari-v0.3.0.zip` | Apple Safari | Safari WebExtension package |

---

### 🔐 Cryptographic Checksums (SHA-256)
Refer to [`SHA256SUMS.txt`](SHA256SUMS.txt) for cryptographic integrity verification.
