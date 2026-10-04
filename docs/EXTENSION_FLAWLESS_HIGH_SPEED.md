# AURORA Extension — Flawless + High-Speed Audit

Source of truth: `apps/aurora-extension/` mirrored to
`chrome / brave / edge / firefox / safari` via `scripts/sync_extension.py`.
Engine: `crates/aurora-wasm/` (`aurora_wasm_bg.wasm` ~341 KB) + `core/downloader.js`.

Pipeline:

```
click / downloads.onCreated -> background/service-worker.js
  -> core/downloader.js probe(HEAD -> Range 0-0 -> GET)
  -> WasmDownloadTask.fetchRange x N (parallel Range GET)
  -> finalizeAndSave (Blob assemble -> checksum -> chrome.downloads.download)
  -> offscreen/offscreen.html+js + core/idb-store.js (MV3 Blob URL bridge)
  -> popup/popup.js (500ms poll render) + content/interceptor.js (toast)
```

## Speed fixes applied

1. **Zero-copy range buffers** — `core/downloader.js:fetchRange`
   Before: `new Uint8Array(expectedLength)` upfront + `buffer.set(chunk)` per
   network chunk (1 big malloc + 1 copy per chunk, OOM on GB segments).
   After: `parts.push(value)` zero-copy + `new Blob(parts)` per segment.
   Final `new Blob(Blobs)` is native, peak JS heap ~1 chunk, no upfront
   contiguous malloc. Verified by `extension.test.mjs` 15/15 pass.

2. **Honest scheduler passthrough** — `core/downloader.js:run`
   Before: forced `'fixed'` for all non-single types, UI claimed ECT.
   After: passes real `schedulerType` (`aurora-ect`/`largest-segment`/`fixed`)
   to `AuroraWasmEngine`. Browser streams stay non-overlapping (no mid-flight
   fetch resize); adaptive split metadata now reaches UI via `get_snapshot()`.

3. **Bounded probe latency** — `core/downloader.js:probe`
   Added 15s abort per HEAD/Range/GET probe linked to task abort.
   Prevents hung probe stalling queue on dead/slow servers.

4. **Popup perf correctness** — `popup/popup.js`
   - Merge key `id` not `url||id`: duplicate-URL re-downloads no longer collapse.
   - Desktop `fetch(.../api/history*)` now 4s `AbortController` timeout (was
     unbounded, could hang popup).
   - `togglePauseResume` now handles `Probing/Queued` -> pause and
     `Failed` -> retry (retry button was dead).

5. **Content-script cost** — `content/interceptor.js`
   - `document.body` null guards in `triggerFlyingFileToTop` /
     `showTopRightDownloadPanel` (crashed on early `document_start` clicks).
   - Extension list now runtime-built from `settings.interceptExtensions`
     instead of hardcoded regex (options page was dead setting).

## Flawless fixes applied

| # | File | Bug | Fix |
|---|------|-----|-----|
| 1 | `manifest.json` x6 + `scripts/build_extension.sh` | `background` uses `chrome.tabs.query(active)` but no `activeTab`; `url` may be empty on restricted pages | Added `"activeTab"` to all 6 manifests + build templates |
| 2 | `background/service-worker.js` | `internalDownloadUrls` Set never evicted (blob URL per download) | `registerInternalUrl()` LRU cap 200 + 5min TTL |
| 3 | `background/service-worker.js` | `alarms.create(periodInMinutes: 0.5)` clamped by Chrome (min ~1) | Changed to `1` |
| 4 | `core/downloader.js:triggerChromeDownload` | Required `document !== undefined` for `URL.createObjectURL`; Firefox/Safari background (no offscreen doc) always failed | Try `URL.createObjectURL` in any context first, offscreen+IDB only as Chromium SW fallback |
| 5 | `options/options.js` | `connections: parseInt(...)` can be `NaN`; `schedulerType` unvalidated; empty ext list saved | Clamp 1–32, whitelist scheduler, drop empty list |
| 6 | `offscreen` / `idb-store.js` | Blob leak if browser save interrupted before `REVOKE_OBJECT_URL` | Documented; `watchBrowserDownload` cleanup runs on `complete` + `interrupted` (no change needed, verified in test `browser completion controls status`) |

## Verified

```bash
npm test --prefix apps/aurora-extension
# 15 pass, 0 fail (~2s mock + ~1.9s real HTTP 4MiB: 1-conn ~3 MiB/s, 4-conn ~11.7 MiB/s)

python3 scripts/sync_extension.py --check
# All five browser copies match the shared extension source.
```

WASM `pkg/` (`aurora_wasm.js`, `aurora_wasm_bg.wasm`, `.d.ts`) unchanged;
`Firefox` manifest keeps `background.scripts` + `gecko.id`, others keep
`service_worker` + `offscreen` permission.

## Known limits (not changed, by design)

- **Full-file RAM/Blob**: all range Blobs + final `Blob` + optional
  `blob.arrayBuffer()` for checksum live in memory. 10GB+ files need
  OPFS/File System Access streaming — biggest next win.
- **Checksum duplicates memory**: `compute_sha256/blake3(data: &[u8])`
  requires whole file. Needs incremental `Hasher::update()` exposed to JS.
- **Resume restarts**: `resume()` resets `downloadedBytes/chunks` and
  re-fetches all ranges. True byte-offset resume needs persisted chunk bitmap.
- **No live ECT split in browser**: `fetch` streams can't be resized
  mid-flight; ECT adaptation currently only affects scheduling metadata.
  Desktop native engine does real splitting.
- **Badge blink `setInterval(700ms)`**: killed on SW suspend; alarm (1min)
  is the reliable keepalive. Blink is cosmetic only.
- **History cap**: `persistTasks()` keeps last 200 snapshots; rehydrated
  `Downloading/Probing/Assembling/Verifying/Queued` -> `Paused` on restart.

## Next high-speed roadmap

1. OPFS staging + `File System Access` streaming assemble (constant RAM).
2. Incremental WASM hash API (`blake3_new/update/finalize`) + `blob.stream()` pipe.
3. Adaptive concurrency: shrink/grow `connections` from EWMA/RTT instead of fixed 8.
4. Range resume bitmap in `chrome.storage.local` + `If-Range` revalidate.
5. Replace 500ms popup poll with `chrome.runtime.onMessage` push + `requestAnimationFrame` batch.
6. Restrict content script (`match_about_blank:false`, skip `all_frames` where safe) and debounce toast DOM.

## Files touched

- `apps/aurora-extension/{manifest.json,background/service-worker.js,core/downloader.js,popup/popup.js,content/interceptor.js,options/options.js}` + 5 mirrored copies
- `scripts/build_extension.sh` (Firefox/Safari manifest templates)
