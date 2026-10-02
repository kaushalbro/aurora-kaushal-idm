# AURORA Kaushal IDM v0.1.0 - Official Multi-Platform Release

🚀 **AURORA Kaushal IDM** is a high-performance, next-generation download accelerator engineered in Rust with WebAssembly. Featuring **Adaptive Estimated Completion Time (ECT)** dynamic work stealing, zero-copy positioned disk I/O, cryptographic hash verification, and universal browser extension integration.

---

## 📥 Download Links & Binaries

### 🖥️ Desktop Applications
| Operating System | Package | Size | SHA-256 Checksum |
| :--- | :--- | :--- | :--- |
| **Linux (Debian / Ubuntu)** | [`aurora-kaushal-idm_0.1.0_amd64.deb`](aurora-kaushal-idm_0.1.0_amd64.deb) | **6.0 MB** | `ef1839c726dc705cf20b374b799f20d024bed19a796a01fe7cbb6ef788f111ce` |
| **Windows 10 / 11** | [`aurora-kaushal-idm-v0.1.0-windows-x64.zip`](aurora-kaushal-idm-v0.1.0-windows-x64.zip) | **6.9 MB** | `e470482dd5812e7b879b8a2d21da558afaa00687645b26f91dcbd8a616f42a58` |
| **Apple macOS** | [`aurora-kaushal-idm-v0.1.0-macos.zip`](aurora-kaushal-idm-v0.1.0-macos.zip) | **8.0 MB** | `ad593926337a05ebb9e7c4b8c5b02de1c95c227d4ab87e9dfc3669f0c6964267` |
| **Linux (Generic / Arch / Fedora)** | [`aurora-kaushal-idm-v0.1.0-linux-x86_64.tar.gz`](aurora-kaushal-idm-v0.1.0-linux-x86_64.tar.gz) | **8.0 MB** | `f71ba9f0fac8acf492a894f9ca0f37318ab71c4546676bbc99a04de193f54c50` |

---

### 🌐 Universal Browser Extensions
| Browser | Package | SHA-256 Checksum |
| :--- | :--- | :--- |
| **Google Chrome** | [`aurora-chrome-v0.1.0.zip`](aurora-chrome-v0.1.0.zip) | `dcb136ac2362f7ca68ea00c5006f16ded582434f1aa7c753c4f64345da1d1ddc` |
| **Brave Browser** | [`aurora-brave-v0.1.0.zip`](aurora-brave-v0.1.0.zip) | `dcb136ac2362f7ca68ea00c5006f16ded582434f1aa7c753c4f64345da1d1ddc` |
| **Microsoft Edge** | [`aurora-edge-v0.1.0.zip`](aurora-edge-v0.1.0.zip) | `dcb136ac2362f7ca68ea00c5006f16ded582434f1aa7c753c4f64345da1d1ddc` |
| **Mozilla Firefox** | [`aurora-firefox-v0.1.0.xpi`](aurora-firefox-v0.1.0.xpi) | `392a702f75fade965a6b4e8ba0086990175c977e66e30089e9d262e023f8e3ff` |
| **Apple Safari** | [`aurora-safari-v0.1.0.zip`](aurora-safari-v0.1.0.zip) | `a942b1872e7ea3da92cd455f197c0baa605b6a68a1d8cc58e71679949009249f` |

---

## ✨ Key Features & Capabilities

- ⚡ **AURORA ECT Engine**: Adaptive dynamic work stealing splits straggler segments and accelerates downloads up to 32 concurrent TCP streams.
- 🔄 **Real-Time Two-Way Sync**: Native sync bridge connects Browser WebExtensions with Desktop GUI on `127.0.0.1:28282`.
- 🛡️ **Cryptographic Integrity Verification**: Real-time streaming SHA-256 and BLAKE3 hash validation.
- 📂 **Multi-Select & Bulk Operations**: Bulk file deletion from disk or list with interactive confirmation and single-click download link copying.
- 🧹 **Clean Installation & Uninstallation**: Installers automatically terminate old running instances and delete previous caches before installing fresh files.

---

## 🛠️ Quick Installation Guide

### Linux (.deb)
```bash
sudo dpkg -i aurora-kaushal-idm_0.1.0_amd64.deb
```

### Windows
1. Extract `aurora-kaushal-idm-v0.1.0-windows-x64.zip`.
2. Double-click `install.bat` to install and create a Desktop shortcut (or run `aurora-gui.exe` directly).

### macOS
1. Extract `aurora-kaushal-idm-v0.1.0-macos.zip`.
2. Run `./install.sh` or drag `AURORA Kaushal IDM.app` into `/Applications`.
