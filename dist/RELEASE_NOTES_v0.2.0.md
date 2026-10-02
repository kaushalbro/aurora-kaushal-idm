# AURORA Kaushal IDM v0.2.0 - Official Multi-Platform Release

AURORA Kaushal IDM is an ultra-high performance, next-generation download accelerator engineered in pure Rust. It features Adaptive Estimated Completion Time (ECT) dynamic work stealing, zero-copy positioned disk I/O, cryptographic hash verification, system tray integration, autostart on system boot, and universal browser download interception.

---

## 📥 Desktop Applications & Packages

| Operating System | Package | Size | SHA-256 Checksum |
| :--- | :--- | :--- | :--- |
| **Linux (Debian / Ubuntu)** | [`aurora-kaushal-idm_0.2.0_amd64.deb`](aurora-kaushal-idm_0.2.0_amd64.deb) | **6.1 MB** | `d8eb09d4d645a7b18441669336d28799ff2e552550851a19f01ecf4ba884164c` |
| **Windows (10 / 11 Installer)** | [`aurora-kaushal-idm-v0.2.0-setup.exe`](aurora-kaushal-idm-v0.2.0-setup.exe) | **5.4 MB** | `5db4620bc018795e1f98f184460b0bdbeea4e3d95247b5fa99d6c79d00a3930f` |
| **Windows (10 / 11 Portable)** | [`aurora-kaushal-idm-v0.2.0-windows-x64.zip`](aurora-kaushal-idm-v0.2.0-windows-x64.zip) | **6.9 MB** | `8f43d2300f82184f99dc99211ec7918177790e7eb0c4cb8181d0eaa48ce8249c` |
| **Apple macOS (10.15+)** | [`aurora-kaushal-idm-v0.2.0-macos.zip`](aurora-kaushal-idm-v0.2.0-macos.zip) | **8.1 MB** | `5aa2cc66f8e276b49742c7f0a37b093a7062ec7ecf0e4d066ef0f1fefc4a5d4b` |
| **Linux (Generic / Portable)** | [`aurora-kaushal-idm-v0.2.0-linux-x86_64.tar.gz`](aurora-kaushal-idm-v0.2.0-linux-x86_64.tar.gz) | **8.1 MB** | `bbbe674369bf19543c469e97801b5f8058d83af10fde15e74850f46207479593` |

---

## 🌐 Universal Browser Extensions

| Browser | Package | SHA-256 Checksum |
| :--- | :--- | :--- |
| **Google Chrome** | [`aurora-chrome-v0.2.0.zip`](aurora-chrome-v0.2.0.zip) | `b528f8efde5a76160f767bbc2e65844d049e38b9e87ca03464f6dd64d8bcb230` |
| **Brave Browser** | [`aurora-brave-v0.2.0.zip`](aurora-brave-v0.2.0.zip) | `b528f8efde5a76160f767bbc2e65844d049e38b9e87ca03464f6dd64d8bcb230` |
| **Microsoft Edge** | [`aurora-edge-v0.2.0.zip`](aurora-edge-v0.2.0.zip) | `b528f8efde5a76160f767bbc2e65844d049e38b9e87ca03464f6dd64d8bcb230` |
| **Mozilla Firefox** | [`aurora-firefox-v0.2.0.xpi`](aurora-firefox-v0.2.0.xpi) | `e3ea84df7134d265cd3b0fe69183aad897accbef58b55f18664e67a21560ac24` |
| **Apple Safari** | [`aurora-safari-v0.2.0.zip`](aurora-safari-v0.2.0.zip) | `420f6152c18efe88071794323ba0f4d9c3a8ff10ea191359761e767aef880408` |

---

## ✨ New in Version 0.2.0

- 🚀 **Auto-Start on Boot / Reboot**: Seamless cross-platform system startup integration for Linux (`.desktop`), Windows (Registry Run), and macOS (`LaunchAgents`).
- 🎛️ **Native System Tray with Quick Action Menu**: Minimize to tray on close, quick Add URL, Pause All, Resume All, and Auto-Start toggling.
- 🌐 **Universal Browser Interception**:
  - Deep link handler support (`aurora://` and `auroradl://`).
  - Single-instance IPC socket forwards arguments from any browser or terminal directly into the running instance.
  - Universal Clipboard Sniffer detects copied download URLs across any browser (Chrome, Firefox, Safari, Edge, Brave, Opera, Tor, Arc) and pre-populates the Quick Add modal with automated remote capability probing.
- 🎨 **Modern React 19 + Tailwind CSS Frontend**: Single-row clean toolbar, live telemetry and bandwidth stats integrated in sidebar, collapsible 60s speed graph, and cyber dark theme.
- 🛡️ **Cryptographic Verification**: Real-time streaming SHA-256 and BLAKE3 hash validation.

---

## 🛠️ Quick Installation Guide

### Linux (.deb)
```bash
sudo dpkg -i aurora-kaushal-idm_0.2.0_amd64.deb
```

### Windows
1. Run `aurora-kaushal-idm-v0.2.0-setup.exe` or extract `aurora-kaushal-idm-v0.2.0-windows-x64.zip`.
2. Launch AURORA IDM from the Start Menu or Desktop.

### macOS
1. Extract `aurora-kaushal-idm-v0.2.0-macos.zip`.
2. Run `./install.sh` or drag `AURORA IDM.app` into `/Applications`.
