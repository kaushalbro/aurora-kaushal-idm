# AURORA Kaushal IDM v0.1.0 - Official Multi-Platform Release

AURORA Kaushal IDM is a high-performance, next-generation download accelerator engineered in pure Rust. It features Adaptive Estimated Completion Time (ECT) dynamic work stealing, zero-copy positioned disk I/O, cryptographic hash verification, and native desktop synchronization.

---

## Desktop Applications & Packages

| Operating System | Package | Size | SHA-256 Checksum |
| :--- | :--- | :--- | :--- |
| **Linux (Debian / Ubuntu)** | `aurora-kaushal-idm_0.1.0_amd64.deb` | **6.0 MB** | `ef1839c726dc705cf20b374b799f20d024bed19a796a01fe7cbb6ef788f111ce` |
| **Windows (10 / 11)** | `aurora-kaushal-idm-v0.1.0-windows-x64.zip` | **6.9 MB** | `e470482dd5812e7b879b8a2d21da558afaa00687645b26f91dcbd8a616f42a58` |
| **Apple macOS (10.15+)** | `aurora-kaushal-idm-v0.1.0-macos.zip` | **8.0 MB** | `ad593926337a05ebb9e7c4b8c5b02de1c95c227d4ab87e9dfc3669f0c6964267` |
| **Linux (Generic / Portable)** | `aurora-kaushal-idm-v0.1.0-linux-x86_64.tar.gz` | **8.0 MB** | `f71ba9f0fac8acf492a894f9ca0f37318ab71c4546676bbc99a04de193f54c50` |

---

## Key Features & Capabilities

- **AURORA ECT Engine**: Adaptive dynamic work stealing splits straggler segments and accelerates downloads up to 32 concurrent TCP streams.
- **Direct Positioned Storage**: Writes directly to target disk offsets with zero-copy I/O and disk backpressure control.
- **Crash-Safe Recovery Journal**: State journal (`.aurora.state`) with remote HTTP validators guarantees seamless resume across power interruptions.
- **Cryptographic Integrity Verification**: Real-time streaming SHA-256 and BLAKE3 hash validation.
- **Clean Installation & Uninstallation**: Automated scripts ensure old running processes and legacy files are cleanly removed before installation and on complete uninstallation.

---

## Quick Installation Guide

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
