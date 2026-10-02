# AURORA Download Manager

AURORA is a high-performance, lightweight, cross-platform download manager engineered in pure Rust for Linux (Ubuntu, Debian, Fedora, Arch, Zorin OS), Windows (10/11), and macOS.

Unlike traditional download managers that rely on fixed chunk partitioning or naive largest-range splitting, AURORA is built around an experimental research scheduler designed around Expected Completion Time (ECT), rate-proportional segment splitting, adaptive segment sizing, adaptive concurrency control, and disk backpressure.

---

## Key Capabilities & Highlights

- **Pure Rust Architecture**: Zero Electron, zero Chromium, zero webviews. Native GUI powered by `egui` and `eframe` with near-instant startup and minimal memory footprint.
- **Interchangeable Research Schedulers**:
  - **Baseline Single-Stream**: Single continuous byte range.
  - **Fixed Segmentation**: Static partition into N equal chunks.
  - **Largest-Segment Splitting**: IDM-style baseline (idle workers split the largest active range in half).
  - **AURORA Expected Completion Time (ECT)**: Straggler-aware work stealing, rate-proportional assignment `R * (ra / (ra + rb))`, and adaptive segment duration target clamping.
- **Direct Positioned Storage Engine**: Asynchronously writes bytes directly to target file offsets using platform-native `pwrite` (Linux/POSIX) and `seek_write` (Windows) without creating temporary segment files on disk.
- **Disk Backpressure & Memory Bounding**: Memory queues remain strictly bounded even on 100+ GB downloads by throttling network ingestion when disk committed write rate drops.
- **Crash-Safe Atomic Recovery Journal**: Write-flush-rename journal (`.aurora.state`) with strict remote validator checks (ETag, Last-Modified, Content-Length) to guarantee data integrity across crashes or power loss.
- **Discrete Simulation & Mock Test Server**: Includes an in-process mock HTTP server with fault injection and an analytical discrete-event simulator for deterministic benchmarking.
- **Two-Way Browser Extension Sync**: Built-in background bridge on `127.0.0.1:28282` provides real-time status and two-way history synchronization between Desktop and WebExtensions (Chrome, Brave, Edge, Firefox, Safari).

---

## Installation & Downloads

Pre-built, ultra-compressed binary packages are available on the GitHub Releases page:
https://github.com/kaushalbro/aurora-kaushal-idm/releases

### 1. Linux (Ubuntu, Debian, Linux Mint, Zorin OS)

Download the official `.deb` installer and install via package manager:

```bash
# Install package
sudo dpkg -i aurora-kaushal-idm_0.2.0_amd64.deb

# Launch application
aurora-gui
```

### 2. Windows (10 / 11)

1. Download `aurora-kaushal-idm-v0.2.0-windows-x64.zip`.
2. Extract the archive.
3. Double-click `install.bat` for an automated clean install and Desktop shortcut creation, or launch `aurora-gui.exe` directly as a portable application.

### 3. macOS (10.15+)

1. Download `aurora-kaushal-idm-v0.2.0-macos.zip`.
2. Extract the archive.
3. Run `./install.sh` to install `AURORA Kaushal IDM.app` into `/Applications`, or drag the application bundle to `/Applications`.

### 4. Linux Generic / Portable (Arch, Fedora, openSUSE)

```bash
tar -xzf aurora-kaushal-idm-v0.2.0-linux-x86_64.tar.gz
cd linux-portable
./install.sh
```

---

## Workspace Structure

```
AURORA/
├── Cargo.toml                  # Workspace manifest
├── crates/
│   ├── aurora-core/            # Core models, ByteRange arithmetic, events, typed errors, platform paths
│   ├── aurora-storage/         # PositionedWriter, platform fallocate/pwrite, backpressure, SHA256/BLAKE3
│   ├── aurora-http/            # Reqwest/rustls client, HEAD/Range 0-0 probe, stream transport, retry backoff
│   ├── aurora-metrics/         # EWMA throughput, Jacobson RTT tracker, RTT inflation, multi-objective utility
│   ├── aurora-recovery/        # Atomic state journal, resume validation engine
│   ├── aurora-scheduler/       # SingleStream, Fixed, LargestSegment, and AURORA-ECT schedulers
│   ├── aurora-history/         # SQLite history database & decayed host performance profiles
│   ├── aurora-benchmark/      # Pure-Rust mock server, discrete simulator, automated benchmark harness
│   └── aurora-wasm/           # WebAssembly bindings for WebExtension core engine
├── apps/
│   ├── aurora-cli/             # Command-line interface for download, benchmark, simulate, and test server
│   ├── aurora-gui/             # Native egui desktop interface with real-time segment visualizer & sync server
│   ├── aurora-extension/       # Universal WebExtension base
│   ├── aurora-chrome-extension # Google Chrome WebExtension distribution
│   ├── aurora-brave-extension  # Brave Browser WebExtension distribution
│   ├── aurora-edge-extension   # Microsoft Edge WebExtension distribution
│   ├── aurora-firefox-extension# Mozilla Firefox WebExtension distribution
│   └── aurora-safari-extension # Apple Safari WebExtension distribution
├── scripts/
│   ├── build_extension.sh      # Compiles WebAssembly and packages browser extensions
│   ├── package_desktop.sh      # Builds and compresses .deb, Windows .exe, macOS bundle, and .tar.gz
│   └── upload_to_github_release.py # Automated GitHub Release publisher
└── docs/                       # Technical documentation
```

---

## Quick Start

### Building from Source

Prerequisites:
- Rust 1.75+ (install via `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)

```bash
# Clone the repository
git clone https://github.com/kaushalbro/aurora-kaushal-idm.git
cd aurora-kaushal-idm

# Build release binaries
cargo build --release
```

### CLI Usage

```bash
# Download a file with default AURORA-ECT scheduling:
cargo run --release --bin aurora -- download https://speed.hetzner.de/100MB.bin

# Download using IDM-style Largest-Segment Splitting for comparison:
cargo run --release --bin aurora -- download https://speed.hetzner.de/100MB.bin --scheduler largest-segment --connections 8

# Run automated multi-scheduler benchmarks:
cargo run --release --bin aurora -- benchmark https://speed.hetzner.de/100MB.bin --runs 3 --format json
```

### Discrete-Event Scheduler Simulator

```bash
# Simulate 1 GB download with heterogeneous workers (100 MB/s, 30 MB/s, 10 MB/s):
cargo run --release --bin aurora -- simulate --workers 100,30,10 --file-size-mb 1024
```

### Launch Desktop GUI

```bash
cargo run --release --bin aurora-gui
```

---

## Clean Reinstallation & Uninstallation

All packaging scripts include lifecycle hooks to ensure that previous installations and running instances are cleanly terminated before new versions are installed:

- **Debian (`.deb`)**:
  - `preinst` & `prerm`: Automatically stops running processes and cleans temporary files.
  - `postrm`: Completely removes icons, desktop shortcuts, and configurations upon package purge.
- **Windows**:
  - `install.bat`: Terminates existing instances, purges old installation directories, and sets up fresh shortcuts.
  - `uninstall.bat`: Stops running processes, removes `%LOCALAPPDATA%\Programs\AuroraIDM`, and deletes Desktop/Start-Menu shortcuts.
- **macOS / Linux Portable**:
  - `install.sh` and `uninstall.sh` handle process termination and clean directory replacements.

---

## License

This project is dual-licensed under either:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
