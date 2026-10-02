# AURORA Kaushal IDM v0.1.0 - Official Multi-Platform Release

AURORA Kaushal IDM is a high-performance, next-generation download accelerator engineered in pure Rust. It features Adaptive Estimated Completion Time (ECT) dynamic work stealing, zero-copy positioned disk I/O, cryptographic hash verification, and native desktop synchronization.

---

## Desktop Applications & Packages

| Operating System | Package | Size | SHA-256 Checksum |
| :--- | :--- | :--- | :--- |
| **Linux (Debian / Ubuntu)** | `aurora-kaushal-idm_0.1.0_amd64.deb` | **6.0 MB** | `b8bd641706d56b9a538fdecf79418647e027a94ef7a2ea70ef58d653187bcfbb` |
| **Windows (10 / 11)** | `aurora-kaushal-idm-v0.1.0-windows-x64.zip` | **6.9 MB** | `341ec286ea18c8f4c9457e51b882c5497b25335e833a7f0dbbc700ef03ba075e` |
| **Apple macOS (10.15+)** | `aurora-kaushal-idm-v0.1.0-macos.zip` | **16.0 MB** | `14454a415a4c2cb6ffb38b005e9b6eabee19ab91ee535b3488e4a435b7fc2138` |
| **Linux (Generic / Portable)** | `aurora-kaushal-idm-v0.1.0-linux-x86_64.tar.gz` | **8.0 MB** | `b3bfbdcab2790be70bd3d27f9f248e98511474c3f4608260b9761eabca183724` |

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
2. Double-click `install.bat` to install and create Desktop and Start Menu shortcuts with the official icon, or run `setup.vbs` for silent installation.

### macOS
1. Extract `aurora-kaushal-idm-v0.1.0-macos.zip`.
2. Run `./install.sh` or drag `AURORA Kaushal IDM.app` into `/Applications`.
