# AURORA Download Manager

**AURORA** is a high-performance, lightweight, cross-platform download manager engineered in **pure Rust** for Linux (Ubuntu, Debian, Fedora, Arch, Zorin OS) and Windows (10/11).

Unlike traditional download managers that rely on fixed chunk partitioning or naive largest-range splitting, AURORA is built around an experimental research scheduler designed around **Expected Completion Time (ECT)**, **rate-proportional segment splitting**, **adaptive segment sizing**, **adaptive concurrency control**, and **disk backpressure**.

---

## Key Capabilities & Highlights

- **Pure Rust Architecture**: Zero Electron, zero Chromium, zero webviews. Native GUI powered by `egui` and `eframe` with near-instant startup and minimal memory footprint.
- **Interchangeable Research Schedulers**:
  1. **Baseline Single-Stream**: Single continuous byte range.
  2. **Fixed Segmentation**: Static partition into $N$ equal chunks.
  3. **Largest-Segment Splitting**: IDM-style baseline (idle workers split the largest active range in half).
  4. **AURORA Expected Completion Time (ECT)**: Straggler-aware work stealing, rate-proportional assignment $R \cdot \frac{r_a}{r_a + r_b}$, and adaptive segment duration target clamping.
- **Direct Positioned Storage Engine**: Asynchronously writes bytes directly to target file offsets without creating temporary segment files on disk.
- **Disk Backpressure & Memory Bounding**: Memory queues remain strictly bounded even on 100+ GB downloads by throttling network ingestion when disk committed write rate drops.
- **Crash-Safe Atomic Recovery Journal**: Write-flush-rename journal (`.aurora.state`) with strict remote validator checks (`ETag`, `Last-Modified`, `Content-Length`) to guarantee data integrity across crashes or power loss.
- **Discrete Simulation & Mock Test Server**: Includes an in-process mock HTTP server with fault injection and an analytical discrete-event simulator for deterministic benchmarking.

---

## Workspace Structure

```text
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
│   └── aurora-benchmark/      # Pure-Rust mock server, discrete simulator, automated benchmark harness
├── apps/
│   ├── aurora-cli/             # Command-line interface for download, benchmark, simulate, and test server
│   └── aurora-gui/             # Native egui desktop interface with real-time segment visualizer
└── docs/                       # Technical documentation
```

---

## Quick Start

### Build & Run CLI

```bash
# Download a file with default AURORA-ECT scheduling:
cargo run --release --bin aurora -- download https://speed.hetzner.de/100MB.bin

# Download using IDM-style Largest-Segment Splitting for comparison:
cargo run --release --bin aurora -- download https://speed.hetzner.de/100MB.bin --scheduler largest-segment --connections 8

# Run automated multi-scheduler benchmarks:
cargo run --release --bin aurora -- benchmark https://speed.hetzner.de/100MB.bin --runs 3 --format json
```

### Run Discrete-Event Scheduler Simulator

```bash
# Simulate 1 GB download with heterogeneous workers (100 MB/s, 30 MB/s, 10 MB/s):
cargo run --release --bin aurora -- simulate --workers 100,30,10 --file-size-mb 1024
```

### Launch Desktop GUI

```bash
cargo run --release --bin aurora-gui
```

---

## License

Dual-licensed under MIT or Apache-2.0.
