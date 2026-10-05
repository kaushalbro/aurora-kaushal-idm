/**
 * AURORA WebAssembly Download Coordinator & Stream Engine
 */
import initWasm, { AuroraWasmEngine, compute_sha256, compute_blake3 } from '../pkg/aurora_wasm.js';
import { saveBlob } from './idb-store.js';

let wasmInitialized = false;
let wasmLoading = null;
let offscreenCreating = null;

export async function ensureWasmLoaded() {
  if (wasmInitialized) return;
  if (!wasmLoading) {
    wasmLoading = (async () => {
      try {
        const wasmUrl = globalThis.chrome?.runtime?.getURL
          ? chrome.runtime.getURL('pkg/aurora_wasm_bg.wasm')
          : new URL('../pkg/aurora_wasm_bg.wasm', import.meta.url);
        const res = await fetch(wasmUrl);
        if (!res.ok) throw new Error(`WASM HTTP ${res.status}`);
        await initWasm({ module_or_path: await res.arrayBuffer() });
        wasmInitialized = true;
      } catch (err) {
        console.warn('[AURORA] WebAssembly load fallback:', err);
      }
    })().finally(() => { wasmLoading = null; });
  }
  await wasmLoading;
}

class RangeUnsupportedError extends Error {}

/**
 * Ensure Chrome Offscreen Document is created for DOM-based Blob Object URL handling
 */
export async function ensureOffscreenDocument() {
  // Firefox and Safari do not use or require offscreen documents
  const g = typeof globalThis !== 'undefined' ? globalThis : self;
  const chromeApi = g.chrome;
  if (!chromeApi || !chromeApi['offscreen']) return;

  const runtimeApi = chromeApi['runtime'];
  const offscreenApi = chromeApi['offscreen'];
  const offscreenUrl = runtimeApi?.getURL ? runtimeApi.getURL('offscreen/offscreen.html') : '';

  if (runtimeApi && typeof runtimeApi['getContexts'] === 'function') {
    try {
      const existingContexts = await runtimeApi['getContexts']({
        contextTypes: ['OFFSCREEN_DOCUMENT'],
        documentUrls: [offscreenUrl]
      });
      if (existingContexts && existingContexts.length > 0) return;
    } catch (_) {}
  }

  if (offscreenCreating) {
    await offscreenCreating;
    return;
  }

  offscreenCreating = (async () => {
    try {
      if (typeof offscreenApi['createDocument'] === 'function') {
        await offscreenApi['createDocument']({
          url: 'offscreen/offscreen.html',
          reasons: ['BLOBS'],
          justification: 'Assemble downloaded files and trigger browser downloads'
        });
        console.log('[AURORA] Chrome Offscreen Document successfully created.');
      }
    } catch (err) {
      if (!err.message?.includes('Only a single offscreen document may be created')) {
        console.warn('[AURORA] Could not create offscreen document:', err);
      }
    } finally {
      offscreenCreating = null;
    }
  })();

  await offscreenCreating;
}


import mimeDb from './mime-db.js';

export const MIME_EXTENSION_MAP = Object.fromEntries(
  Object.entries(mimeDb)
    .filter(([, info]) => info.extensions?.length)
    .map(([mime, info]) => [
      mime,
      info.extensions,
    ])
);

export function getMimeExtension(mimeType) {
  if (!mimeType) return null;
  const clean = mimeType.split(';')[0].trim().toLowerCase();
  const extList = MIME_EXTENSION_MAP[clean];
  if (Array.isArray(extList) && extList.length > 0) {
    return extList[0];
  }
  return null;
}

export class WasmDownloadTask {
  constructor(id, url, filename, options = {}) {
    this.id = id;
    this.url = url;
    this.finalUrl = url;
    this.filename = filename || null;
    this.connections = Math.min(32, Math.max(1, Number.parseInt(options.connections, 10) || 8));
    this.schedulerType = options.schedulerType || 'aurora-ect';
    this.checksum = options.checksum || null;
    this.hashAlgo = options.hashAlgo || 'sha256';
    this.onStatusChange = options.onStatusChange || null;

    this.totalBytes = null;
    this.downloadedBytes = 0;
    this.acceptsRanges = false;
    this.etag = null;
    this.mimeType = null;
    this._status = 'Queued'; // Queued, Probing, Downloading, Assembling, Verifying, Completed, Paused, Failed, Cancelled
    this.errorMessage = null;

    this.wasmEngine = null;
    this.chunks = [];
    this.abortController = new AbortController();
    this.isPaused = false;
    this.activeWorkers = 0;
    this.startTime = Date.now();
    this.speedBytesPerSec = 0;
    this.etaSeconds = null;
    this.downloadId = null;
    this.running = null;
    this.speedSamples = [];
    // Timing display fields are sourced from the Rust WASM engine snapshot
    // (started_at_ms / ended_at_ms / expected_end_ms / remaining_seconds /
    // elapsed_seconds / average_speed_bytes_per_sec). JS fallbacks below are
    // only used when WASM could not load.
    this.completedAtMs = null;
  }

  get status() {
    return this._status;
  }

  set status(val) {
    this._status = val;
    if (typeof this.onStatusChange === 'function') {
      try {
        this.onStatusChange(this, val);
      } catch (_) {}
    }
  }

  async probe() {
    this.status = 'Probing';
    let res = null;
    // Bound probe latency: a hung HEAD/Range probe must not stall the task.
    const probeTimeoutMs = 15000;
    const withProbeTimeout = (signal) => {
      if (signal?.aborted) return signal;
      const ctrl = new AbortController();
      const timer = setTimeout(() => ctrl.abort(new Error('Probe timeout')), probeTimeoutMs);
      const forward = () => { clearTimeout(timer); };
      signal?.addEventListener?.('abort', () => { clearTimeout(timer); ctrl.abort(signal.reason); }, { once: true });
      // Link task-level abort so pause/cancel still wins.
      this.abortController.signal.addEventListener('abort', () => { clearTimeout(timer); ctrl.abort(this.abortController.signal.reason); }, { once: true });
      ctrl.signal.addEventListener('abort', forward, { once: true });
      return ctrl.signal;
    };
    try {
      // Step 1: Probe server with HEAD or GET
      try {
        res = await fetch(this.url, {
          method: 'HEAD',
          redirect: 'follow',
          credentials: 'include',
          signal: withProbeTimeout(this.abortController.signal)
        });
      } catch (_) {}

      // Fallback to GET probe with Range: bytes=0-0 if HEAD was blocked (Cloudflare / S3 403/405/400) or incomplete
      if (!res || !res.ok || (!res.headers.get('content-length') && !res.headers.get('accept-ranges'))) {
        try {
          res = await fetch(this.url, {
            method: 'GET',
            headers: { 'Range': 'bytes=0-0' },
            redirect: 'follow',
            credentials: 'include',
            signal: this.abortController.signal
          });
        } catch (e) {
          // If range get fails, try simple GET probe
          res = await fetch(this.url, {
            method: 'GET',
            redirect: 'follow',
            credentials: 'include',
            signal: this.abortController.signal
          });
        }
      }

      if (!res || !res.ok) {
        throw new Error(`Server returned HTTP ${res?.status || 'Error'} ${res?.statusText || ''}`);
      }

      this.finalUrl = res.url || this.url;
      this.mimeType = (res.headers.get('content-type') || '').split(';')[0].trim().toLowerCase();

      // Check range support
      const acceptRanges = (res.headers.get('accept-ranges') || '').toLowerCase();
      const contentRange = res.headers.get('content-range');
      this.acceptsRanges = acceptRanges === 'bytes' || Boolean(contentRange) || res.status === 206;

      // Extract Content-Length
      if (contentRange) {
        const match = contentRange.match(/\/(\d+)/);
        if (match) {
          this.totalBytes = parseInt(match[1], 10);
        }
      }
      
      if (!this.totalBytes) {
        const cl = res.headers.get('content-length');
        if (cl) {
          this.totalBytes = parseInt(cl, 10);
        }
      }

      if (res.headers.get('content-encoding') && res.headers.get('content-encoding') !== 'identity') {
        this.acceptsRanges = false;
        this.totalBytes = null; // Fetch exposes decoded bytes, not the encoded Content-Length.
      }
      this.etag = res.headers.get('etag');

      // 1. Try to extract filename from Content-Disposition header (RFC 5987 UTF-8 support)
      const cd = res.headers.get('content-disposition');
      if (cd) {
        const utfMatch = cd.match(/filename\*=UTF-8''([^;]+)/i);
        if (utfMatch && utfMatch[1]) {
          try {
            this.filename = decodeURIComponent(utfMatch[1].trim());
          } catch (_) {
            this.filename = utfMatch[1].trim();
          }
        } else {
          const fnMatch = cd.match(/filename=["']?([^"';]+)["']?/i);
          if (fnMatch && fnMatch[1]) {
            try {
              this.filename = decodeURIComponent(fnMatch[1].trim());
            } catch (_) {
              this.filename = fnMatch[1].trim();
            }
          }
        }
      }

      // 2. Extract filename from URL (stripping query strings & hashes)
      if (!this.filename || !this.filename.includes('.')) {
        try {
          const u = new URL(this.finalUrl);
          const pathname = u.pathname || '';
          const segs = pathname.split('/').filter(Boolean);
          if (segs.length > 0) {
            let lastSeg = decodeURIComponent(segs[segs.length - 1].split('?')[0].split('#')[0]);
            if (lastSeg.includes('.')) {
              this.filename = lastSeg;
            } else if (!this.filename) {
              this.filename = lastSeg;
            }
          }
        } catch (_) {}
      }

      // 3. Infer or correct extension from MIME type if missing or if current extension is a backend script (e.g. .php, .aspx, .jsp, .cgi, .do)
      const mimeExt = getMimeExtension(this.mimeType);
      const isBackendScript = /\.(php|aspx|asp|jsp|cgi|do|action|axd)(?:[?#]|$)/i.test(this.filename || '');
      if (mimeExt) {
        if (isBackendScript && this.filename) {
          this.filename = this.filename.replace(/\.(php|aspx|asp|jsp|cgi|do|action|axd)$/i, '.' + mimeExt);
        } else if (this.filename && !this.filename.includes('.')) {
          this.filename += '.' + mimeExt;
        }
      }

      // 4. Default fallback
      if (!this.filename || this.filename.trim() === '') {
        const ext = mimeExt || 'bin';
        this.filename = `download_${Date.now()}.${ext}`;
      }

      // Sanitize filename against invalid characters and directory traversal
      this.filename = this.filename.replace(/(\.\.[\/\\]|\.\.)/g, '').replace(/[<>:"/\\|?*\x00-\x1F]/g, '_').trim();
      if (this.filename.length > 255) {
        this.filename = this.filename.substring(this.filename.length - 255);
      }

      console.log(`[AURORA] Probed target: filename="${this.filename}", totalBytes=${this.totalBytes}, rangeSupport=${this.acceptsRanges}`);
      return true;
    } catch (err) {
      console.warn('[AURORA] Probe error:', err);
      this.acceptsRanges = false;
      if (!this.filename) this.filename = 'download.bin';
      throw err;
    } finally {
      await res?.body?.cancel().catch(() => {});
    }
  }

  start() {
    if (this.running) return this.running;
    this.running = this.run().finally(() => { this.running = null; });
    return this.running;
  }

  async run() {
    try {
      this.abortController = new AbortController();
      this.isPaused = false;
      this.errorMessage = null;
      this.downloadId = null;
      this.startTime = Date.now();
      this.completedAtMs = null;
      this.downloadedBytes = 0;
      this.chunks = [];
      this.totalBytes = null;
      this.wasmEngine?.free();
      this.wasmEngine = null;
      await Promise.all([ensureWasmLoaded(), this.probe()]);
      if (this.abortController.signal.aborted) return;

      if (!this.acceptsRanges || !this.totalBytes || this.totalBytes === 0) {
        console.log('[AURORA] Range headers not supported or size unknown. Streaming via single connection.');
        await this.downloadSingleStream();
        return;
      }

      this.resetSpeed();
      console.log(`[AURORA] Starting multi-stream download: ${this.totalBytes} bytes, ${this.connections} conns (${this.schedulerType})`);
      this.status = 'Downloading';

      const conns = this.schedulerType === 'single-stream' || this.schedulerType === 'single'
        ? 1 : Math.min(this.connections, this.totalBytes);
      // Browser streams use non-overlapping partitions until live split cancellation is supported.
      // Initialize Rust WebAssembly Scheduler Engine
      // Pass the real scheduler type through: Fixed gives static partitions,
      // ECT / Largest-Segment enable adaptive split metadata for the UI.
      // Browser fetch streams stay non-overlapping (no mid-flight resize),
      // the native desktop engine handles live splitting.
      if (wasmInitialized) {
        try {
          const wasmSchedulerType = this.schedulerType === 'single-stream' || this.schedulerType === 'single'
            ? 'single-stream'
            : this.schedulerType;
          this.wasmEngine = new AuroraWasmEngine(
            this.finalUrl,
            BigInt(this.totalBytes),
            conns,
            wasmSchedulerType
          );
        } catch (e) {
          console.warn('[AURORA] Rust WASM engine init failed, falling back to JS coordinator:', e);
          this.wasmEngine = null;
        }
      }

      // Launch worker streams
      const workerPromises = [];

      if (this.wasmEngine) {
        for (let workerId = 1; workerId <= conns; workerId++) {
          workerPromises.push(this.runWasmWorker(workerId));
        }
      } else {
        // High-speed pure JS multi-stream fallback
        const chunkSize = Math.ceil(this.totalBytes / conns);
        for (let workerId = 1; workerId <= conns; workerId++) {
          const start = (workerId - 1) * chunkSize;
          const end = Math.min(workerId * chunkSize - 1, this.totalBytes - 1);
          if (start <= end) {
            this.activeWorkers++;
            workerPromises.push(this.fetchRange(workerId, workerId, start, end).finally(() => { this.activeWorkers--; }));
          }
        }
      }

      // Abort siblings before restarting; never mix partial ranges with a full response.
      let workerError;
      await Promise.all(workerPromises.map(promise => promise.catch(err => {
        if (!workerError) {
          workerError = err;
          this.abortController.abort();
        }
      })));
      if (workerError) {
        if (this.isPaused) return;
        if (!(workerError instanceof RangeUnsupportedError)) throw workerError;
        this.abortController = new AbortController();
        this.wasmEngine?.free();
        this.wasmEngine = null;
        this.chunks = [];
        this.downloadedBytes = 0;
        this.acceptsRanges = false;
        await this.downloadSingleStream();
        return;
      }

      if (this.isPaused || this.abortController.signal.aborted) {
        return;
      }

      if (this.downloadedBytes !== this.totalBytes) {
        throw new Error(`Incomplete download: ${this.downloadedBytes} of ${this.totalBytes} bytes`);
      }
      await this.finalizeAndSave();
    } catch (err) {
      if (!this.isPaused) {
        this.abortController.abort();
        console.error(`[AURORA] Download task ${this.id} failed:`, err);
        this.errorMessage = err.message || String(err);
        this.status = 'Failed';
        this.speedBytesPerSec = 0;
      }
    } finally {
      this.activeWorkers = 0;
    }
  }

  async runWasmWorker(workerId) {
    this.activeWorkers++;
    try {
      while (!this.isPaused && !this.abortController.signal.aborted) {
        const action = this.wasmEngine?.get_next_action(this.activeWorkers);

        // In browser WASM, each worker handles its assigned non-overlapping initial partition.
        // We only process 'start_segment' to avoid duplicate/overlapping in-flight range fetches.
        if (!action || action.action_type !== 'start_segment') break;

        const start = Number(action.start);
        const end = Number(action.end);
        const segId = action.segment_id;

        if (start > end) break;

        await this.fetchRange(workerId, segId, start, end);
      }
    } catch (err) {
      if (!this.isPaused && !this.abortController.signal.aborted) {
        throw err;
      }
    } finally {
      this.activeWorkers--;
    }
  }

  async fetchRange(workerId, segId, start, end, retries = 3) {
    const rangeHeader = `bytes=${start}-${end}`;
    let lastProgressAt = performance.now();

    let res = null;
    let attempt = 0;

    while (attempt <= retries) {
      try {
        if (this.isPaused || this.abortController.signal.aborted) return;

        res = await fetch(this.finalUrl, {
          method: 'GET',
          headers: { 'Range': rangeHeader, ...(this.etag && !this.etag.startsWith('W/') ? { 'If-Range': this.etag } : {}) },
          credentials: 'include',
          signal: this.abortController.signal
        });

        // Handle HTTP 429 / 503 throttling with exponential backoff & jitter
        if (res.status === 429 || res.status === 503) {
          await res.body?.cancel();
          attempt++;
          if (attempt <= retries) {
            const backoffMs = Math.min(1000 * Math.pow(2, attempt) + Math.random() * 500, 8000);
            console.warn(`[AURORA] Worker #${workerId} rate-limited (HTTP ${res.status}). Retrying in ${Math.round(backoffMs)}ms...`);
            await new Promise(r => setTimeout(r, backoffMs));
            continue;
          }
        }

        break;
      } catch (netErr) {
        attempt++;
        if (attempt <= retries && !this.isPaused && !this.abortController.signal.aborted) {
          const backoffMs = 500 * Math.pow(2, attempt);
          await new Promise(r => setTimeout(r, backoffMs));
          continue;
        }
        throw netErr;
      }
    }

    if (!res || (!res.ok && res.status !== 206)) {
      throw new Error(`HTTP ${res?.status || 'Error'} fetching range ${rangeHeader}`);
    }

    if (res.status === 200) {
      await res.body?.cancel();
      throw new RangeUnsupportedError('Server ignored the range; restarting as a single stream');
    }
    const range = /^bytes (\d+)-(\d+)\/(\d+)$/i.exec(res.headers.get('content-range') || '');
    if (res.status !== 206 || !range || Number(range[1]) !== start ||
        Number(range[2]) !== end || Number(range[3]) !== this.totalBytes ||
        (res.headers.get('content-encoding') && res.headers.get('content-encoding') !== 'identity') ||
        (this.etag && res.headers.get('etag') && res.headers.get('etag') !== this.etag)) {
      await res.body?.cancel();
      throw new Error(`Invalid Content-Range or changed resource for ${rangeHeader}`);
    }

    const reader = res.body.getReader();
    const expectedLength = end - start + 1;
    // Zero-copy part list: avoids one large upfront malloc + one memmove per
    // network chunk. Browser Blob storage backs large segments natively and
    // keeps peak JS heap at ~1 chunk instead of ~1 whole segment.
    const parts = [];
    let bytesReceivedForSeg = 0;

    try {
      while (true) {
        if (this.isPaused || this.abortController.signal.aborted) {
          reader.cancel().catch(() => {});
          return;
        }

        const { done, value } = await reader.read();
        if (done) break;

        if (value) {
          if (value.byteLength > expectedLength - bytesReceivedForSeg) {
            await reader.cancel();
            throw new Error(`Oversized range response for ${rangeHeader}`);
          }
          const toCopy = value.byteLength;
          if (toCopy > 0) {
            // Store the network buffer directly (zero-copy). Reader chunks
            // are freshly allocated and safe to retain.
            parts.push(value);
            bytesReceivedForSeg += toCopy;
            this.downloadedBytes += toCopy;

            const now = performance.now();
            const dt = now - lastProgressAt;
            lastProgressAt = now;
            this.calculateSpeed();
            if (this.wasmEngine) {
              try {
                this.wasmEngine.record_progress(segId, workerId, BigInt(toCopy), dt);
              } catch (_) {}
            }
          }

          if (bytesReceivedForSeg >= expectedLength) {
            reader.cancel().catch(() => {});
            break;
          }
        }
      }
    } catch (readErr) {
      if (!this.isPaused && !this.abortController.signal.aborted) {
        throw readErr;
      }
      return;
    }

    if (bytesReceivedForSeg !== expectedLength) {
      throw new Error(`Truncated range: expected ${expectedLength}, received ${bytesReceivedForSeg}`);
    }
    // Wrap parts in a Blob immediately so large segments are backed by the
    // browser (often disk-spilled) instead of one huge JS Uint8Array.
    this.chunks.push({ start, end, data: new Blob(parts) });
    if (this.wasmEngine) {
      try {
        this.wasmEngine.mark_segment_completed(segId);
      } catch (_) {}
    }
  }

  async downloadSingleStream() {
    this.resetSpeed();
    this.activeWorkers = 1;
    this.status = 'Downloading';
    const res = await fetch(this.finalUrl, {
      credentials: 'include',
      signal: this.abortController.signal
    });
    if (!res.ok) throw new Error(`HTTP ${res.status} ${res.statusText}`);

    const encoded = res.headers.get('content-encoding');
    const length = res.headers.get('content-length');
    this.totalBytes = (!encoded || encoded === 'identity') && length !== null ? Number(length) : null;
    const reader = res.body.getReader();
    const receivedChunks = [];
    let received = 0;

    try {
      while (true) {
        if (this.isPaused || this.abortController.signal.aborted) {
          reader.cancel().catch(() => {});
          return;
        }
        const { done, value } = await reader.read();
        if (done) break;
        if (value) {
          receivedChunks.push(value);
          received += value.byteLength;
          this.downloadedBytes = received;
          this.calculateSpeed();
        }
      }
    } catch (streamErr) {
      if (!this.isPaused && !this.abortController.signal.aborted) {
        throw streamErr;
      }
      return;
    }

    this.activeWorkers = 0;
    if (this.totalBytes !== null && received !== this.totalBytes) throw new Error('Truncated single-stream download');
    this.totalBytes = received;
    this.chunks = [{ start: 0, end: received - 1, data: new Blob(receivedChunks) }];
    await this.finalizeAndSave();
  }

  resetSpeed(now = performance.now()) {
    this.speedSamples = [{ time: now, bytes: this.downloadedBytes }];
    this.speedBytesPerSec = 0;
    this.etaSeconds = null;
  }

  calculateSpeed(now = performance.now()) {
    if (!this.speedSamples.length) this.resetSpeed(now);
    const samples = this.speedSamples;
    const last = samples[samples.length - 1];
    if (now - last.time >= 100) samples.push({ time: now, bytes: this.downloadedBytes });
    while (samples.length > 1 && samples[1].time <= now - 2000) samples.shift();
    const elapsed = (now - samples[0].time) / 1000;
    if (elapsed >= 0.1) {
      this.speedBytesPerSec = Math.max(0, this.downloadedBytes - samples[0].bytes) / elapsed;
    }
    this.etaSeconds = this.totalBytes !== null && this.speedBytesPerSec > 0
      ? Math.max(0, this.totalBytes - this.downloadedBytes) / this.speedBytesPerSec : null;
  }

  async finalizeAndSave() {
    this.status = 'Assembling';
    console.log('[AURORA] Assembling downloaded chunks into final file...');

    this.chunks.sort((a, b) => a.start - b.start);
    let offset = 0;
    for (const chunk of this.chunks) {
      const size = chunk.data.byteLength ?? chunk.data.size;
      if (chunk.start !== offset || size !== chunk.end - chunk.start + 1) throw new Error('Incomplete or overlapping file ranges');
      offset += size;
    }
    if (offset !== this.totalBytes) throw new Error('Incomplete file');
    const dataParts = this.chunks.map(c => c.data);
    const blob = new Blob(dataParts, { type: this.mimeType || 'application/octet-stream' });

    // Immediate memory cleanup: release raw chunks array
    this.chunks = [];

    if (this.checksum) {
      if (!wasmInitialized) throw new Error('Checksum verification requires the WebAssembly engine');
      this.status = 'Verifying';
      const buffer = new Uint8Array(await blob.arrayBuffer());
      const hash = this.hashAlgo === 'blake3' ? compute_blake3(buffer) : compute_sha256(buffer);
      if (hash.toLowerCase() !== this.checksum.trim().toLowerCase()) {
        this.status = 'Failed';
        this.errorMessage = `Checksum mismatch! Expected: ${this.checksum}, Actual: ${hash}`;
        throw new Error(this.errorMessage);
      }
    }

    // Rust owns the end timestamp: stamp completion in the WASM engine
    // before handing the file to the browser.
    try {
      if (this.wasmEngine && typeof this.wasmEngine.mark_completed === 'function') {
        this.wasmEngine.mark_completed();
      }
    } catch (_) {}

    await this.triggerChromeDownload(blob);
  }

  async triggerChromeDownload(blob) {
    this.speedBytesPerSec = 0;
    this.etaSeconds = null;

    // 1. Fast path: create a Blob URL directly in this context.
    // Works in popups, content-adjacent pages, Firefox background pages,
    // and any context with URL.createObjectURL. MV3 Chromium service
    // workers lack DOM URL creation, so they fall through to (2).
    if (typeof URL !== 'undefined' && typeof URL.createObjectURL === 'function') {
      try {
        const objectUrl = URL.createObjectURL(blob);
        try {
          await chrome.runtime.sendMessage({ action: 'REGISTER_INTERNAL_URL', url: objectUrl });
        } catch (_) {}

        if (typeof chrome !== 'undefined' && chrome.downloads?.download) {
          this.downloadId = await chrome.downloads.download({
            url: objectUrl,
            filename: this.filename || 'download.bin',
            saveAs: false
          });
        }

        this.watchBrowserDownload(() => URL.revokeObjectURL(objectUrl));
        return;
      } catch (domErr) {
        console.warn('[AURORA] Direct Blob object URL failed, trying Offscreen bridge:', domErr);
      }
    }

    // 2. Manifest V3 Service Worker (Chromium):
    // Save blob to IndexedDB -> Request Offscreen Document to create Blob Object URL -> Trigger chrome.downloads.download() in Service Worker
    try {
      console.log(`[AURORA] Storing blob for task ${this.id} (${blob.size} bytes) in IndexedDB...`);
      await saveBlob(this.id, blob);

      console.log('[AURORA] Ensuring Offscreen Document is active...');
      await ensureOffscreenDocument();

      const res = await chrome.runtime.sendMessage({
        action: 'CREATE_OBJECT_URL',
        taskId: this.id,
        filename: this.filename || 'download.bin',
        mimeType: this.mimeType || 'application/octet-stream'
      });

      if (!res || !res.success || !res.objectUrl) {
        throw new Error(res?.error || 'Failed to create Object URL in Offscreen document');
      }

      const objectUrl = res.objectUrl;

      // Register internal URL so service worker doesn't intercept its own download
      try {
        await chrome.runtime.sendMessage({ action: 'REGISTER_INTERNAL_URL', url: objectUrl });
      } catch (_) {}

      // Trigger download using the Service Worker's chrome.downloads API
      if (typeof chrome !== 'undefined' && chrome.downloads?.download) {
        this.downloadId = await chrome.downloads.download({
          url: objectUrl,
          filename: this.filename || 'download.bin',
          saveAs: false
        });
        console.log(`[AURORA] Successfully triggered browser download id=${this.downloadId} for task ${this.id}`);
      } else {
        throw new Error('chrome.downloads.download API is not available');
      }

      // Cleanup listener / timer
      const cleanup = () => {
        chrome.runtime.sendMessage({ action: 'REVOKE_OBJECT_URL', taskId: this.id, objectUrl }).catch(() => {});
      };

      this.watchBrowserDownload(cleanup);
    } catch (swErr) {
      console.error(`[AURORA] Failed to finalize download for task ${this.id}:`, swErr);
      this.status = 'Failed';
      this.errorMessage = swErr.message || String(swErr);
      throw swErr;
    }
  }

  watchBrowserDownload(cleanup) {
    if (!Number.isInteger(this.downloadId)) throw new Error('Browser did not accept the download');
    let finished = false;
    const finish = state => {
      if (finished || (state !== 'complete' && state !== 'interrupted')) return;
      finished = true;
      chrome.downloads.onChanged.removeListener(onChanged);
      cleanup();
      if (state === 'interrupted') this.errorMessage = 'Browser interrupted saving the file';
      if (state === 'complete' && this.completedAtMs == null) this.completedAtMs = Date.now();
      if (this.status !== 'Cancelled') this.status = state === 'complete' ? 'Completed' : 'Failed';
    };
    const onChanged = delta => {
      if (delta.id === this.downloadId) finish(delta.state?.current);
    };
    chrome.downloads.onChanged.addListener(onChanged);
    // Covers completion between download() resolving and listener registration.
    chrome.downloads.search({ id: this.downloadId }).then(items => finish(items[0]?.state)).catch(() => {});
  }

  pause() {
    if (!['Queued', 'Probing', 'Downloading'].includes(this.status)) return;
    this.isPaused = true;
    this.status = 'Paused';
    this.speedBytesPerSec = 0;
    this.abortController.abort();
  }

  async resume() {
    if (this.status === 'Paused' || this.status === 'Failed') {
      await this.running;
      if (this.status === 'Cancelled') return;
      return this.start(); // In-memory transfers restart cleanly; no duplicate chunks.
    }
  }

  cancel() {
    this.isPaused = true;
    this.speedBytesPerSec = 0;
    this.status = 'Cancelled';
    this.abortController.abort();
    if (Number.isInteger(this.downloadId)) chrome.downloads.cancel(this.downloadId).catch(() => {});
    this.chunks = [];
  }

  getSnapshot() {
    if (this.status === 'Downloading') this.calculateSpeed();
    else { this.speedBytesPerSec = 0; this.etaSeconds = null; }
    let segments = [];
    let wasmTiming = null;
    try {
      const snap = this.wasmEngine?.get_snapshot();
      if (snap) {
        segments = snap.segments || [];
        wasmTiming = snap;
      }
    } catch (_) {}
    const progressPct = this.totalBytes && this.totalBytes > 0 ? (this.downloadedBytes / this.totalBytes) * 100 : 0;
    // Timing display comes from the Rust WASM engine (started_at_ms,
    // ended_at_ms, expected_end_ms, remaining_seconds, elapsed_seconds,
    // average_speed_bytes_per_sec). JS fallbacks apply only without WASM.
    const startedAtMs = wasmTiming?.started_at_ms ?? this.startTime ?? Date.now();
    const endedAtMs = wasmTiming?.ended_at_ms ?? this.completedAtMs ?? null;
    const remainingSeconds = wasmTiming?.remaining_seconds ?? this.etaSeconds ?? null;
    const expectedEndMs = wasmTiming?.expected_end_ms
      ?? (this.etaSeconds != null ? Date.now() + this.etaSeconds * 1000 : null);
    const elapsedSeconds = wasmTiming?.elapsed_seconds
      ?? Math.max(0, (Date.now() - startedAtMs) / 1000);
    const totalTimeSeconds = endedAtMs != null
      ? Math.max(0, (endedAtMs - startedAtMs) / 1000) : null;
    const averageSpeedBps = wasmTiming?.average_speed_bytes_per_sec
      ?? (totalTimeSeconds ? this.downloadedBytes / totalTimeSeconds : 0);
    return {
      id: this.id,
      downloadId: this.downloadId,
      filename: this.filename || 'download.bin',
      url: this.finalUrl || this.url,
      connections: this.connections,
      schedulerType: this.schedulerType,
      checksum: this.checksum,
      hashAlgo: this.hashAlgo,
      totalBytes: this.totalBytes,
      downloadedBytes: this.downloadedBytes,
      status: this.status,
      errorMessage: this.errorMessage,
      speedBytesPerSec: this.speedBytesPerSec,
      speedMbps: (this.speedBytesPerSec * 8) / 1_000_000,
      etaSeconds: this.etaSeconds,
      progressPct: isFinite(progressPct) ? progressPct : 0,
      segments,
      activeConnections: this.status === 'Downloading' ? this.activeWorkers : 0,
      startedAtMs,
      endedAtMs,
      expectedEndMs,
      remainingSeconds,
      elapsedSeconds,
      totalTimeSeconds,
      averageSpeedBps: Number.isFinite(averageSpeedBps) ? averageSpeedBps : 0
    };
  }
}

