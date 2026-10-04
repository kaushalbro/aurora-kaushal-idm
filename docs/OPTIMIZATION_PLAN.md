# AURORA optimization plan

Audit date: 2026-10-04. This plan separates implemented repairs from proposed architecture changes. No internet-speed improvement is claimed without measurements on the affected host and connection.

## Current system

- Rust workspace: shared HTTP transport (Reqwest 0.12/Rustls, HTTP/2), scheduler, metrics, positioned storage, recovery, history, and benchmark crates.
- Two desktop clients: egui (`apps/aurora-gui`) and React 19/Tauri 2 (`apps/aurora-desktop`). The README's claim that there are no webviews does not describe the Tauri client.
- Browser extension: Manifest V3 background coordinator, JavaScript fetch streams, Rust/WASM scheduler and checksums, in-memory ranges assembled into a Blob, IndexedDB handoff to a Chromium offscreen document, then browser download API.
- Five browser-specific source copies derive from `apps/aurora-extension`. Keep fixes synchronized while preserving each browser's manifest.
- Desktop bridge uses loopback HTTP on port 28282; popup routing and history sync are separate from the extension transfer engine.

## Repairs in this change

1. **Current aggregate speed:** use total byte deltas over a roughly two-second monotonic clock window, excluding probing. Previously each chunk was divided by elapsed time since its request began, then that per-chunk/per-worker estimate overrode the aggregate rate. This underreported sustained throughput. Speed now decays during stalls and is zero in inactive states.
2. **Units:** binary byte rates are KiB/s and MiB/s; hover text shows decimal Mbps. For example, 100 Mbps equals 12.5 MB/s or about 11.92 MiB/s before overhead. A real low transfer rate still displays KiB/s.
3. **Range integrity:** check HTTP 206 and exact Content-Range, total size, validators when provided, encoding, and segment length. Cancel unused probe bodies. If a server ignores Range, drain/abort sibling workers and restart once as a full stream. Never save incomplete or overlapping ranges as successful downloads.
4. **Browser scheduling:** use fixed, parallel, non-overlapping partitions. The prior browser ECT integration shortened the scheduler's range without shortening the already-running fetch. Live splitting remains a desktop feature until coordinated cancellation/offset updates are implemented and tested. Single-stream now creates exactly one range. Tiny native partitions cannot underflow.
5. **Lifecycle correctness:** propagate worker failures, wait for aborted workers before restart, clear old chunks/counters, share WASM initialization, verify checksums on single streams, wait for browser completion before reporting success, retain object URLs until the browser finishes, and resolve IndexedDB writes only after transaction commit. Offscreen messages have only one response owner. Rehydrated paused tasks can restart from the URL.

## Ordered follow-up work

| Priority | Work | Acceptance criteria |
| --- | --- | --- |
| P0 | Audit the native writer path in `aurora-scheduler/src/engine.rs`: it currently notifies some bytes before disk commit, ignores final flush/sync failures, and may write a received chunk past a rebalanced segment boundary. Route writes through one bounded, acknowledged writer interface. | Inject disk-full, short writes, stream drops, overlapping split boundaries and process termination. No false success; final hash matches; recovery journal never leads committed bytes. |
| P0 | Durable browser transfer state and correct MV3 lifetime behavior. Persist range manifests with validators and committed offsets; recover interrupted saves by browser download ID. Move long-running transfer coordination to a supported document/worker or native backend, with an explicit background-lifecycle design. | Download with popup closed, suspend/restart service worker, restart browser, cancel during assembly, interrupt file save. Recover or show a clear resumable failure with no silent loss. Alarms alone are not a persistence guarantee. |
| P1 | Replace whole-file `Uint8Array` allocation and final full-file IndexedDB copy with bounded chunks, OPFS storage in a dedicated worker, backpressure and incremental checksums. Feature-detect and provide an explicit fallback. | Download a 10 GB test file with a fixed memory ceiling (initial target: <128 MiB transfer buffers), handle quota exhaustion, verify exact hash, and clean abandoned storage. |
| P1 | Per-origin measured concurrency. Start conservatively, increase only while useful byte throughput improves, back off on 429/503 with Retry-After and cancellation-aware delays. Account for browser-controlled HTTP/2 multiplexing. | Compare 1/4/8/16 connections under fixed per-host and per-connection limits, high RTT, lossy links, and shared bandwidth. Record medians/p95 over at least five runs, retries, memory and total transferred bytes. Avoid regression >5% against the best baseline. |
| P1 | Unify all add/intercept/context-menu paths with desktop routing preferences. Consider native messaging with explicit host registration and allowed extension IDs. Audit bridge authentication/origin checks before expanding its capabilities. | Exactly one transfer per click, no duplicate browser interception, clear fallback if desktop is unavailable, authenticated control channel. |
| P2 | Review dependency upgrades in isolated PRs. Evaluate migration from Reqwest 0.12 to its current stable line and update frontend tooling only with reproducible lockfiles/builds. Retain HTTP/2 as a baseline; make HTTP/3 a measured opt-in experiment. | Rust unit/integration tests, WASM build, desktop production build, browser matrix, and before/after throughput data. No broad upgrade justified solely by version number. |
| P2 | Generate distribution copies and archives from one source in CI; choose a maintained primary desktop UI and update architecture documentation. | Mirror parity check, browser manifest validation, reproducible package checksums, no manual edits to generated browser code. |

## Validation and reproduction

- `node tests/extension.test.mjs` exercises the actual checked-in WASM with controlled HTTP responses, rate calculations, exact output bytes, ignored/invalid/truncated ranges, single-stream mode, checksum failure, and pause/restart. Browser APIs are mocked; this does not prove browser lifecycle compatibility.
- `cargo test -p aurora-scheduler -p aurora-http -p aurora-metrics --lib --offline` covers the relevant native units, including empty/tiny partitioning.
- `python3 scripts/sync_extension.py --check` checks shared source parity across all browser directories.
- Reload the unpacked browser extension after applying fixes. Use the same public URL in browser-only mode, native desktop mode and the browser's default downloader. Record file size/hash, elapsed transfer time, displayed byte rate, network Mbps, browser version and connection count. Avoid comparing cached transfers to uncached ones.
- Keep the existing release archives unchanged until a versioned release is prepared. The browser source folders are the updated load-unpacked artifacts.

## Results for this change

- 15 extension tests passed against the rebuilt WASM, including a real loopback HTTP transfer, JS fallback, and IndexedDB commit/abort handling.
- 14 native unit tests passed across scheduler, HTTP and metrics crates; release WASM compilation and binding generation succeeded.
- Controlled 4 MiB fixture, server paced at 64 KiB every 20 ms **per connection**: one connection took 1.322 s (3.02 MiB/s measured; 3.08 MiB/s peak displayed); four took 0.343 s (11.66 MiB/s measured; 11.78 MiB/s peak displayed). These demonstrate aggregation and concurrency under the fixture's conditions, not a before/after internet speedup. Peak rolling speed and whole-transfer average need not be identical.
- All five shared browser copies match; JavaScript syntax and diff whitespace checks passed.
- No live installed-browser, service-worker suspension, physical-network, or multi-GB memory test was performed. Browser save events/IndexedDB APIs were mocked. Native desktop binaries and published release archives were not rebuilt.

## Technology references

- [Chrome service worker lifecycle](https://developer.chrome.com/docs/extensions/develop/concepts/service-workers/lifecycle): background workers can terminate; durable state must survive them.
- [HTTP range requests](https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/Range_requests): response validation and If-Range semantics.
- [OPFS](https://developer.mozilla.org/en-US/docs/Web/API/File_System_API/Origin_private_file_system) and [FileSystemSyncAccessHandle](https://developer.mozilla.org/en-US/docs/Web/API/FileSystemSyncAccessHandle): worker-based local file access for the proposed storage redesign.
- [Chrome native messaging](https://developer.chrome.com/docs/extensions/develop/concepts/native-messaging): registered native hosts and extension permissions for the proposed desktop transport.
- [Reqwest documentation](https://docs.rs/reqwest/latest/reqwest/): HTTP/3 remains documented as unstable with additional opt-in. A new protocol is not automatically a speed fix.
