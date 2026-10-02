/**
 * AURORA WebAssembly Download Coordinator & Stream Engine
 */
import initWasm, { AuroraWasmEngine, compute_sha256, compute_blake3 } from '../pkg/aurora_wasm.js';
import { saveBlob, deleteBlob } from './idb-store.js';

let wasmInitialized = false;
let offscreenCreating = null;

export async function ensureWasmLoaded() {
  if (!wasmInitialized) {
    try {
      const wasmUrl = typeof chrome !== 'undefined' && chrome.runtime?.getURL
        ? chrome.runtime.getURL('pkg/aurora_wasm_bg.wasm')
        : './pkg/aurora_wasm_bg.wasm';
      const res = await fetch(wasmUrl);
      const bytes = await res.arrayBuffer();
      await initWasm({ module_or_path: bytes });
      wasmInitialized = true;
      console.log('[AURORA] Rust WebAssembly Engine initialized with ArrayBuffer.');
    } catch (err) {
      console.warn('[AURORA] WebAssembly load fallback:', err);
    }
  }
}

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


// MIME type to extension dictionary
const MIME_EXTENSION_MAP = {
  'video/mp4': 'mp4',
  'video/webm': 'webm',
  'video/x-matroska': 'mkv',
  'video/quicktime': 'mov',
  'video/x-msvideo': 'avi',
  'video/3gpp': '3gp',
  'audio/mpeg': 'mp3',
  'audio/mp4': 'm4a',
  'audio/ogg': 'ogg',
  'audio/wav': 'wav',
  'audio/flac': 'flac',
  'application/zip': 'zip',
  'application/x-zip-compressed': 'zip',
  'application/x-rar-compressed': 'rar',
  'application/x-7z-compressed': '7z',
  'application/x-tar': 'tar',
  'application/gzip': 'gz',
  'application/x-bzip2': 'bz2',
  'application/x-xz': 'xz',
  'application/x-iso9660-image': 'iso',
  'application/pdf': 'pdf',
  'application/octet-stream': 'bin',
  'application/x-msdownload': 'exe',
  'application/vnd.android.package-archive': 'apk'
};

export class WasmDownloadTask {
  constructor(id, url, filename, options = {}) {
    this.id = id;
    this.url = url;
    this.finalUrl = url;
    this.filename = filename || null;
    this.connections = options.connections || 8;
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
    try {
      // Step 1: Probe server with HEAD or GET
      let res = null;
      try {
        res = await fetch(this.url, {
          method: 'HEAD',
          redirect: 'follow',
          signal: this.abortController.signal
        });
      } catch (_) {}

      // Fallback to GET probe with Range: bytes=0-0 if HEAD was blocked (Cloudflare / S3 403/405/400) or incomplete
      if (!res || !res.ok || (!res.headers.get('content-length') && !res.headers.get('accept-ranges'))) {
        try {
          res = await fetch(this.url, {
            method: 'GET',
            headers: { 'Range': 'bytes=0-0' },
            redirect: 'follow',
            signal: this.abortController.signal
          });
        } catch (e) {
          // If range get fails, try simple GET probe
          res = await fetch(this.url, {
            method: 'GET',
            redirect: 'follow',
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

      // 3. Infer extension from MIME type if missing
      if (this.filename && !this.filename.includes('.') && this.mimeType && MIME_EXTENSION_MAP[this.mimeType]) {
        this.filename += '.' + MIME_EXTENSION_MAP[this.mimeType];
      }

      // 4. Default fallback
      if (!this.filename || this.filename.trim() === '') {
        const ext = (this.mimeType && MIME_EXTENSION_MAP[this.mimeType]) ? MIME_EXTENSION_MAP[this.mimeType] : 'bin';
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
    }
  }

  async start() {
    try {
      await ensureWasmLoaded();
      this.abortController = new AbortController();
      this.isPaused = false;
      this.errorMessage = null;
      this.startTime = Date.now();

      await this.probe();
      if (this.abortController.signal.aborted) return;

      if (!this.acceptsRanges || !this.totalBytes || this.totalBytes === 0) {
        console.log('[AURORA] Range headers not supported or size unknown. Streaming via single connection.');
        await this.downloadSingleStream();
        return;
      }

      console.log(`[AURORA] Starting multi-stream download: ${this.totalBytes} bytes, ${this.connections} conns (${this.schedulerType})`);
      this.status = 'Downloading';

      // Initialize Rust WebAssembly Scheduler Engine
      if (wasmInitialized) {
        try {
          this.wasmEngine = new AuroraWasmEngine(
            this.finalUrl,
            BigInt(this.totalBytes),
            this.connections,
            this.schedulerType
          );
        } catch (e) {
          console.warn('[AURORA] Rust WASM engine init failed, falling back to JS coordinator:', e);
          this.wasmEngine = null;
        }
      }

      // Launch worker streams
      const workerPromises = [];
      const conns = this.connections || 8;

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
            workerPromises.push(this.fetchRange(workerId, workerId, start, end));
          }
        }
      }

      await Promise.all(workerPromises);

      if (this.isPaused || this.abortController.signal.aborted) {
        return;
      }

      if (this.downloadedBytes >= this.totalBytes) {
        await this.finalizeAndSave();
      } else if (this.downloadedBytes > 0) {
        // Save whatever was downloaded
        await this.finalizeAndSave();
      }
    } catch (err) {
      if (!this.isPaused && !this.abortController.signal.aborted) {
        console.error(`[AURORA] Download task ${this.id} failed:`, err);
        this.status = 'Failed';
        this.errorMessage = err.message || String(err);
        this.speedBytesPerSec = 0;
      }
    }
  }

  async runWasmWorker(workerId) {
    this.activeWorkers++;
    try {
      while (!this.isPaused && !this.abortController.signal.aborted) {
        let action = null;
        try {
          action = this.wasmEngine ? this.wasmEngine.get_next_action(this.activeWorkers) : null;
        } catch (_) {}

        if (!action || action.action_type === 'do_nothing') {
          // If all bytes downloaded, exit
          if (this.totalBytes && this.downloadedBytes >= this.totalBytes) {
            break;
          }
          // Pause briefly to allow straggler splits
          await new Promise(r => setTimeout(r, 40));
          if (!this.wasmEngine || this.downloadedBytes >= (this.totalBytes || 0)) {
            break;
          }
          continue;
        }

        const start = Number(action.start);
        const end = Number(action.end);
        const segId = action.segment_id;

        if (start > end) break;

        await this.fetchRange(workerId, segId, start, end);
      }
    } catch (err) {
      if (!this.isPaused && !this.abortController.signal.aborted) {
        console.error(`[AURORA] Worker #${workerId} error:`, err);
      }
    } finally {
      this.activeWorkers--;
    }
  }

  async fetchRange(workerId, segId, start, end, retries = 3) {
    const rangeHeader = `bytes=${start}-${end}`;
    const t0 = performance.now();

    let res = null;
    let attempt = 0;

    while (attempt <= retries) {
      try {
        if (this.isPaused || this.abortController.signal.aborted) return;

        res = await fetch(this.finalUrl, {
          method: 'GET',
          headers: { 'Range': rangeHeader, 'Cache-Control': 'no-cache' },
          signal: this.abortController.signal
        });

        // Handle HTTP 429 / 503 throttling with exponential backoff & jitter
        if (res.status === 429 || res.status === 503) {
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

    // Handle servers returning HTTP 200 (ignoring Range header)
    if (res.status === 200 && start > 0) {
      console.warn('[AURORA] Server returned HTTP 200 for sub-range. Falling back to single-stream mode.');
      this.acceptsRanges = false;
      return;
    }

    const reader = res.body.getReader();
    const expectedLength = end - start + 1;
    const segmentBuffer = new Uint8Array(expectedLength);
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
          const toCopy = Math.min(value.byteLength, expectedLength - bytesReceivedForSeg);
          if (toCopy > 0) {
            segmentBuffer.set(value.subarray(0, toCopy), bytesReceivedForSeg);
            bytesReceivedForSeg += toCopy;
            this.downloadedBytes += toCopy;

            const dt = performance.now() - t0;
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

    this.chunks.push({ start, end, data: segmentBuffer.subarray(0, bytesReceivedForSeg) });
    if (this.wasmEngine) {
      try {
        this.wasmEngine.mark_segment_completed(segId);
      } catch (_) {}
    }
  }

  async downloadSingleStream() {
    this.status = 'Downloading';
    const res = await fetch(this.finalUrl, { signal: this.abortController.signal });
    if (!res.ok) throw new Error(`HTTP ${res.status} ${res.statusText}`);

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

    this.status = 'Assembling';
    const blob = new Blob(receivedChunks, { type: this.mimeType || 'application/octet-stream' });
    await this.triggerChromeDownload(blob);
  }

  calculateSpeed() {
    const elapsed = (Date.now() - this.startTime) / 1000;
    if (elapsed > 0.2 && isFinite(elapsed)) {
      const calcSpeed = this.downloadedBytes / elapsed;
      if (isFinite(calcSpeed) && calcSpeed >= 0) {
        this.speedBytesPerSec = calcSpeed;
        if (this.totalBytes && this.speedBytesPerSec > 1024) {
          const remaining = Math.max(this.totalBytes - this.downloadedBytes, 0);
          const eta = remaining / this.speedBytesPerSec;
          this.etaSeconds = isFinite(eta) && eta >= 0 ? eta : null;
        }
      }
    }
  }

  async finalizeAndSave() {
    this.status = 'Assembling';
    console.log('[AURORA] Assembling downloaded chunks into final file...');

    this.chunks.sort((a, b) => a.start - b.start);
    const dataParts = this.chunks.map(c => c.data);
    const blob = new Blob(dataParts, { type: this.mimeType || 'application/octet-stream' });

    // Immediate memory cleanup: release raw chunks array
    this.chunks = [];

    if (this.checksum) {
      this.status = 'Verifying';
      const buffer = new Uint8Array(await blob.arrayBuffer());
      const hash = this.hashAlgo === 'blake3' ? compute_blake3(buffer) : compute_sha256(buffer);
      if (hash.toLowerCase() !== this.checksum.trim().toLowerCase()) {
        this.status = 'Failed';
        this.errorMessage = `Checksum mismatch! Expected: ${this.checksum}, Actual: ${hash}`;
        throw new Error(this.errorMessage);
      }
    }

    await this.triggerChromeDownload(blob);
  }

  async triggerChromeDownload(blob) {
    this.status = 'Completed';
    this.speedBytesPerSec = 0;

    // 1. Direct URL.createObjectURL (if in DOM context or Firefox)
    if (typeof document !== 'undefined' && typeof URL !== 'undefined' && typeof URL.createObjectURL === 'function') {
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

        setTimeout(() => {
          try { URL.revokeObjectURL(objectUrl); } catch (_) {}
        }, 120000);
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

      if (this.downloadId && typeof chrome !== 'undefined' && chrome.downloads?.onChanged) {
        const onChangedListener = (delta) => {
          if (delta.id === this.downloadId && delta.state) {
            if (delta.state.current === 'complete' || delta.state.current === 'interrupted') {
              chrome.downloads.onChanged.removeListener(onChangedListener);
              cleanup();
            }
          }
        };
        chrome.downloads.onChanged.addListener(onChangedListener);
      }

      // Safety fallback cleanup after 2 minutes
      setTimeout(cleanup, 120000);
    } catch (swErr) {
      console.error(`[AURORA] Failed to finalize download for task ${this.id}:`, swErr);
      this.status = 'Failed';
      this.errorMessage = swErr.message || String(swErr);
      throw swErr;
    }
  }

  pause() {
    this.isPaused = true;
    this.status = 'Paused';
    this.speedBytesPerSec = 0;
    this.abortController.abort();
  }

  resume() {
    if (this.status === 'Paused' || this.status === 'Failed') {
      this.start();
    }
  }

  cancel() {
    this.isPaused = true;
    this.status = 'Cancelled';
    this.abortController.abort();
    this.chunks = [];
  }

  getSnapshot() {
    if (this.wasmEngine) {
      try {
        const snap = this.wasmEngine.get_snapshot();
        return {
          id: this.id,
          downloadId: this.downloadId,
          filename: this.filename || 'download.bin',
          url: this.finalUrl || this.url,
          totalBytes: this.totalBytes,
          downloadedBytes: this.downloadedBytes,
          status: this.status,
          errorMessage: this.errorMessage,
          speedBytesPerSec: isFinite(snap.smoothed_speed_bytes_per_sec) ? snap.smoothed_speed_bytes_per_sec : this.speedBytesPerSec,
          speedMbps: isFinite(snap.smoothed_speed_mbps) ? snap.smoothed_speed_mbps : 0,
          etaSeconds: snap.eta_seconds || this.etaSeconds,
          progressPct: snap.progress_pct || (this.totalBytes ? (this.downloadedBytes / this.totalBytes) * 100 : 0),
          segments: snap.segments || [],
          activeConnections: this.activeWorkers
        };
      } catch (_) {}
    }

    const progressPct = this.totalBytes && this.totalBytes > 0 ? (this.downloadedBytes / this.totalBytes) * 100 : 0;
    return {
      id: this.id,
      downloadId: this.downloadId,
      filename: this.filename || 'download.bin',
      url: this.finalUrl || this.url,
      totalBytes: this.totalBytes,
      downloadedBytes: this.downloadedBytes,
      status: this.status,
      errorMessage: this.errorMessage,
      speedBytesPerSec: this.speedBytesPerSec,
      speedMbps: (this.speedBytesPerSec * 8) / (1024 * 1024),
      etaSeconds: this.etaSeconds,
      progressPct: isFinite(progressPct) ? progressPct : 0,
      segments: [],
      activeConnections: this.activeWorkers
    };
  }
}

