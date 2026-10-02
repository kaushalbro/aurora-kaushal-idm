# AURORA Architecture & Systems Design

This document details the architectural principles, threading model, memory boundaries, and asynchronous dataflow within the AURORA Download Manager.

---

## 1. System Pipeline Overview

```text
               URL / Input
                   │
                   ▼
       [ Capability & Range Probe ]  ─── HEAD / Range: bytes=0-0
                   │
                   ▼
       [ Recovery / Journal Check ]  ─── Invalidate if ETag changed
                   │
                   ▼
     [ Positioned Storage Preallocation ] ── Linux fallocate / Windows SetEndOfFile
                   │
                   ▼
         [ Scheduler Selection ]
         ├── SingleStream
         ├── FixedSegment (N chunks)
         ├── LargestSegment (50/50 IDM baseline)
         └── AuroraECT (Expected Completion Time)
                   │
                   ▼
       ┌───────────────────────┐
       │ Async Worker Network  │ ◄─── Reqwest / Rustls Streams
       └───────────┬───────────┘
                   │ Bounded Channel & Backpressure Check
                   ▼
       ┌───────────────────────┐
       │ PositionedFileWriter  │ ─── Direct OS Offset Writes (pwrite/FileExt)
       └───────────┬───────────┘
                   │ Periodic Atomic Sync (Write -> Sync -> Rename)
                   ▼
         [ .aurora.state Journal ]
                   │
                   ▼
       [ Final Integrity Verification ] ── SHA-256 / BLAKE3 Checksums
                   │
                   ▼
              Finalized File
```

---

## 2. Decoupled Workspace Design

The application is structured into 8 modular crates:

1. **`aurora-core`**:
   - Holds core domain types (`ByteRange`, `Segment`, `WorkerStats`, `ServerCapabilities`).
   - Declares platform-agnostic paths (`platform/linux.rs`, `platform/windows.rs`, `platform/common.rs`).
   - Defines event bus broadcast channels (`DownloadEvent`).
2. **`aurora-storage`**:
   - `PositionedWriter` writes directly to file offsets without intermediate temp files.
   - `BackpressureController` monitors in-flight memory bytes and throttles network ingress when storage flush speed slows.
   - Platform modules implement zero-fill / fallocate preallocation and positioned I/O.
3. **`aurora-http`**:
   - Connection pool management using `reqwest` and `rustls-tls`.
   - Capability probing detecting `Accept-Ranges`, `Content-Length`, `ETag`, and protocol version.
   - Range stream fetching with accurate TTFB (Time to First Byte) RTT measurement.
   - Retry policy handling transient connection resets, timeouts, and `429 Too Many Requests` (respecting `Retry-After`).
4. **`aurora-metrics`**:
   - EWMA rate smoothing for transfer rates and latency.
   - Jacobson RTT variance and RTT inflation monitoring.
   - Multi-factor utility model for evaluating concurrency decisions.
5. **`aurora-recovery`**:
   - Atomic disk journaling (`save_journal`, `load_journal`).
   - ETag and timestamp validator checks.
6. **`aurora-scheduler`**:
   - Implementations of SingleStream, Fixed, LargestSegment, and AURORA-ECT schedulers.
   - Adaptive concurrency controller.
   - Download engine runtime coordinator.
7. **`aurora-history`**:
   - SQLite persistent storage for completed downloads and host profiles with exponential time decay.
8. **`aurora-benchmark`**:
   - Built-in Mock HTTP Server simulating variable bandwidth limits, latency jitter, and HTTP faults.
   - Discrete-event simulation and multi-iteration benchmark runner.

---

## 3. Storage & Bounded Memory Model

Traditional download managers often download separate `.part0`, `.part1`, ... `.partN` files and concatenate them upon completion. This has two major drawbacks:
1. Double disk I/O during finalization (copying hundreds of gigabytes).
2. Disk space exhaustion (requiring $2\times$ the file size).

AURORA preallocates the single final file on disk at startup. Network workers write directly to their designated byte ranges using platform-native positioned writes (`FileExt::write_all_at` on Linux, `FileExt::seek_write` on Windows).

To prevent memory leaks or unbounded buffer growth on fast network connections paired with slow storage:
- Chunks pass through a bounded channel.
- In-flight uncommitted memory bytes are tracked atomically.
- When in-flight memory exceeds `disk_queue_limit_bytes` (default: 64 MB), workers pause reading from network sockets until disk writes drain below 50% capacity.

---

## 4. Cross-Platform Separation Strategy

Platform-specific logic is explicitly isolated into separate modules:

```text
crates/aurora-storage/src/platform/
├── mod.rs      # Cross-platform dispatcher trait
├── linux.rs    # Linux fallocate, posix_fadvise, pwrite
├── windows.rs  # Windows SetEndOfFile, seek_write
└── common.rs   # Portable standard library fallback
```

This ensures maximum performance on each operating system while maintaining cross-platform portability.
