# AURORA Kaushal IDM v0.1.0 - Official Multi-Platform Release

AURORA Kaushal IDM is a high-performance, next-generation download accelerator engineered in pure Rust. It features Adaptive Estimated Completion Time (ECT) dynamic work stealing, zero-copy positioned disk I/O, cryptographic hash verification, and native desktop synchronization.

---

## Desktop Applications & Packages

| Operating System | Package | Size | SHA-256 Checksum |
| :--- | :--- | :--- | :--- |
| **Linux (Debian / Ubuntu)** | `aurora-kaushal-idm_0.1.0_amd64.deb` | **6.0 MB** | `32f99b95d7d2f4af994dd0d6f520f533ddb9a87d964b2955b1c94339fc6697ab` |
| **Windows (10 / 11)** | `aurora-kaushal-idm-v0.1.0-windows-x64.zip` | **6.9 MB** | `338606bf8240793eaddb5c11c89bbc0b23ca1c69e8aaeba4950be6641f99d398` |
| **Apple macOS (10.15+)** | `aurora-kaushal-idm-v0.1.0-macos.zip` | **8.0 MB** | `54cd88a7bae4438346f709fe3daeb397fe37fce6abbf443f74822e9a39bdbb5e` |
| **Linux (Generic / Portable)** | `aurora-kaushal-idm-v0.1.0-linux-x86_64.tar.gz` | **8.0 MB** | `61d3b437a4b933472ee7a816109d2e7adda1562470f4558398b00317724833db` |

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
