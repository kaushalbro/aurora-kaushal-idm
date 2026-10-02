# AURORA Crash-Safe State Journal & Recovery System

This document outlines the design and guarantees of the state journal and resume engine in AURORA.

---

## 1. Crash Invariants & Safety Guarantees

When a download is interrupted by network failure, power loss, OS crash, or user cancellation:
1. **Never Append to a Modified File**: The engine must never append partial byte ranges to a remote file whose contents or validator have changed on the origin server.
2. **Atomicity of Checkpoints**: The on-disk journal must never be partially written or left in a corrupt state due to mid-write power failure.
3. **Bounded Journaling Overhead**: Journal checkpoints are throttled by interval (default $2\,\text{s}$) to avoid stalling the async worker pipeline with disk I/O.

---

## 2. Write-Flush-Rename Atomic Journal Sequence

Directly overwriting an existing metadata JSON file (`state.json`) is hazardous because a crash during `write()` leaves a zero-byte or truncated file.

AURORA utilizes POSIX & Windows atomic file replacement:

```text
[ In-Memory ResumeState ]
           │
           ▼
1. Serialize JSON to `.aurora.state.tmp`
           │
           ▼
2. `File::sync_all()` — flush dirty OS buffer cache to physical media
           │
           ▼
3. `fs::rename(".aurora.state.tmp", ".aurora.state")` — atomic directory entry swap
           │
           ▼
[ Transactionally Committed Checkpoint ]
```

If a power failure occurs during step 1 or 2, `.aurora.state` remains 100% intact from the previous valid checkpoint.

---

## 3. Remote Validator Verification

Upon resuming an existing partial download:

```text
[ Load Local `.aurora.state` ]
           │
           ▼
[ Send HEAD / Range Probe to Remote Server ]
           │
           ▼
Check 1: Does server support HTTP Range requests?
         └── NO  ──► Invalidate partial state (Single-stream full restart)
         └── YES ──► Continue
           │
Check 2: Did Content-Length change?
         └── YES ──► Invalidate partial state (Resource resized)
         └── NO  ──► Continue
           │
Check 3: Did ETag change? (Ignoring W/ weak prefixes)
         └── YES ──► Invalidate partial state (Resource replaced)
         └── NO  ──► Continue
           │
Check 4: Did Last-Modified timestamp change?
         └── YES ──► Invalidate partial state
         └── NO  ──► Continue
           │
Check 5: Does destination file exist on local storage?
         └── NO  ──► Invalidate partial state
         └── YES ──► Valid Resume!
```

---

## 4. Segment State Reconciliation

When resume validation succeeds:
- Completed segments remain marked as `Completed`.
- Interrupted segments are queried for their exact committed bytes on disk (`downloaded`), and the unfulfilled sub-range $[a_i + \text{downloaded}_i, b_i]$ is requeued as `Pending`.
- Active schedulers rebalance the remaining unfulfilled byte slices among active connections.
