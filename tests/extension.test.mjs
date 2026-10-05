import assert from 'node:assert/strict';
import { test, afterEach } from 'node:test';
import { readFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { WasmDownloadTask } from '../apps/aurora-extension/core/downloader.js';
import { formatSpeed, formatNetworkSpeed } from '../apps/aurora-extension/core/format.js';

const originalFetch = globalThis.fetch;
afterEach(() => { globalThis.fetch = originalFetch; delete globalThis.chrome; delete globalThis.document; });
const payload = Uint8Array.from({ length: 65539 }, (_, i) => i % 251);
const wasm = await readFile(new URL('../apps/aurora-extension/pkg/aurora_wasm_bg.wasm', import.meta.url));

function installServer({ mode = 'ranges', head = true } = {}) {
  const requests = [];
  let probeCancelled = false;
  globalThis.fetch = async (url, options = {}) => {
    if (String(url).endsWith('.wasm')) return new Response(wasm);
    const range = options.headers?.Range;
    requests.push({ method: options.method || 'GET', range });
    if (options.method === 'HEAD') {
      if (!head) return new Response(null, { status: 405 });
      return new Response(null, { headers: { 'content-length': payload.length, 'accept-ranges': 'bytes' } });
    }
    if (!head && range === 'bytes=0-0') {
      return new Response(new ReadableStream({ cancel() { probeCancelled = true; } }), {
        status: 206, headers: { 'content-range': `bytes 0-0/${payload.length}` }
      });
    }
    if (range && mode !== 'ignored') {
      const [, start, end] = /bytes=(\d+)-(\d+)/.exec(range).map(Number);
      const bytes = payload.slice(start, mode === 'truncated' ? end : end + 1);
      // Yield so every parallel worker starts before the first completes.
      await new Promise(resolve => setTimeout(resolve, 2));
      if (options.signal?.aborted) throw new DOMException('Aborted', 'AbortError');
      return new Response(bytes, { status: 206, headers: {
        'content-range': `bytes ${mode === 'wrong-range' ? start + 1 : start}-${end}/${payload.length}`
      } });
    }
    return new Response(payload, { headers: { 'content-length': payload.length } });
  };
  return { requests, get probeCancelled() { return probeCancelled; } };
}

function task(options = {}) {
  const t = new WasmDownloadTask('test', 'https://example.test/file.bin', 'file.bin', options);
  t.triggerChromeDownload = async blob => { t.saved = new Uint8Array(await blob.arrayBuffer()); t.status = 'Completed'; };
  return t;
}

test('aggregate rate remains accurate over time and falls to zero on a stall', () => {
  const t = task();
  t.totalBytes = 100 * 1024 ** 2;
  t.resetSpeed(0);
  for (let tick = 1; tick <= 100; tick++) {
    t.downloadedBytes = tick * 2 * 1024 ** 2 / 10;
    t.calculateSpeed(tick * 100);
    assert.ok(Math.abs(t.speedBytesPerSec - 2 * 1024 ** 2) < 0.01);
  }
  t.calculateSpeed(12100);
  t.calculateSpeed(14200);
  assert.equal(t.speedBytesPerSec, 0);
  assert.equal(t.etaSeconds, null);
});

test('snapshot uses aggregate bytes, not incorrect per-worker WASM speed', () => {
  const t = task();
  t.status = 'Downloading';
  t.resetSpeed(performance.now() - 1000);
  t.downloadedBytes = 8 * 1024 ** 2;
  t.wasmEngine = { get_snapshot: () => ({ smoothed_speed_bytes_per_sec: 123, segments: [] }) };
  assert.ok(t.getSnapshot().speedBytesPerSec > 7 * 1024 ** 2);
  t.status = 'Paused';
  assert.equal(t.getSnapshot().speedBytesPerSec, 0);
});

test('binary byte units and decimal network Mbps are explicit', () => {
  assert.equal(formatSpeed(2 * 1024 ** 2), '2.00 MiB/s');
  assert.equal(formatSpeed(2048), '2.00 KiB/s');
  assert.equal(formatSpeed(NaN), '0 B/s');
  assert.equal(formatNetworkSpeed(12_500_000), '100.00 Mbps');
});

test('real WASM downloads parallel ranges and assembles exact content', async () => {
  const server = installServer();
  const t = task({ connections: 8 });
  await t.start();
  assert.equal(t.status, 'Completed', t.errorMessage);
  assert.deepEqual(t.saved, payload);
  assert.equal(t.downloadedBytes, payload.length);
  assert.equal(server.requests.filter(r => r.range).length, 8);
});

test('single-stream scheduler requests the complete file once', async () => {
  const server = installServer();
  const t = task({ schedulerType: 'single', connections: 8 });
  await t.start();
  assert.equal(t.status, 'Completed', t.errorMessage);
  assert.deepEqual(t.saved, payload);
  assert.equal(server.requests.filter(r => r.range).length, 1);
});

test('ignored ranges restart once as a complete single stream', async () => {
  const server = installServer({ mode: 'ignored' });
  const t = task();
  await t.start();
  assert.equal(t.status, 'Completed', t.errorMessage);
  assert.deepEqual(t.saved, payload);
  assert.equal(t.downloadedBytes, payload.length);
  assert.equal(server.requests.filter(r => r.method === 'GET' && !r.range).length, 1);
});

for (const mode of ['truncated', 'wrong-range']) {
  test(`${mode} range fails without saving a corrupt file`, async () => {
    installServer({ mode });
    const t = task();
    await t.start();
    assert.equal(t.status, 'Failed');
    assert.equal(t.saved, undefined);
    assert.ok(t.errorMessage);
    assert.equal(t.activeWorkers, 0);
  });
}

test('GET probe response is cancelled before opening download streams', async () => {
  const server = installServer({ head: false });
  const t = task();
  await t.start();
  assert.equal(server.probeCancelled, true);
  assert.deepEqual(t.saved, payload);
});

test('single stream verifies checksum and rejects mismatches', async () => {
  installServer({ mode: 'ignored' });
  const t = task({ checksum: '0'.repeat(64) });
  await t.start();
  assert.equal(t.status, 'Failed');
  assert.match(t.errorMessage, /Checksum mismatch/);
  assert.equal(t.saved, undefined);
  t.checksum = createHash('sha256').update(payload).digest('hex');
  await t.resume();
  assert.equal(t.status, 'Completed', t.errorMessage);
  assert.deepEqual(t.saved, payload);
  assert.equal(t.downloadedBytes, payload.length);
});

test('browser completion controls status and object URL cleanup', async () => {
  const listeners = new Set();
  let cleanups = 0;
  globalThis.chrome = { downloads: {
    onChanged: { addListener: fn => listeners.add(fn), removeListener: fn => listeners.delete(fn) },
    search: async () => [{ state: 'in_progress' }]
  } };
  const t = task();
  t.downloadId = 12;
  t.status = 'Assembling';
  t.watchBrowserDownload(() => cleanups++);
  await Promise.resolve();
  assert.equal(t.status, 'Assembling');
  assert.equal(cleanups, 0);
  for (const listener of listeners) listener({ id: 12, state: { current: 'complete' } });
  assert.equal(t.status, 'Completed');
  assert.equal(cleanups, 1);
  assert.equal(listeners.size, 0);
});

test('pause followed immediately by resume waits for aborted workers', async () => {
  installServer();
  const t = task();
  const running = t.start();
  await new Promise(resolve => setTimeout(resolve, 1));
  t.pause();
  await t.resume();
  await running;
  assert.equal(t.status, 'Completed', t.errorMessage);
  assert.deepEqual(t.saved, payload);
  assert.equal(t.downloadedBytes, payload.length);
});

test('real HTTP streams transfer exact bytes with measured aggregate throughput', async t => {
  const { createServer } = await import('node:http');
  const content = Uint8Array.from({ length: 4 * 1024 ** 2 }, (_, i) => i % 251);
  const server = createServer((req, res) => {
    if (req.method === 'HEAD') {
      res.writeHead(200, { 'content-length': content.length, 'accept-ranges': 'bytes' });
      res.end();
      return;
    }
    const match = /bytes=(\d+)-(\d+)/.exec(req.headers.range || '');
    const start = match ? Number(match[1]) : 0;
    const end = match ? Number(match[2]) : content.length - 1;
    res.writeHead(match ? 206 : 200, {
      'content-length': end - start + 1,
      ...(match ? { 'content-range': `bytes ${start}-${end}/${content.length}` } : {})
    });
    let offset = start;
    const timer = setInterval(() => {
      const next = Math.min(end + 1, offset + 64 * 1024);
      res.write(content.subarray(offset, next));
      offset = next;
      if (offset > end) { clearInterval(timer); res.end(); }
    }, 20);
    res.on('close', () => clearInterval(timer));
  });
  await new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(0, '127.0.0.1', resolve);
  });
  try {
    globalThis.fetch = originalFetch;
    const results = [];
    for (const connections of [1, 4]) {
      const download = task({ connections });
      download.url = `http://127.0.0.1:${server.address().port}/file.bin`;
      let peakRate = 0;
      const poll = setInterval(() => { peakRate = Math.max(peakRate, download.getSnapshot().speedBytesPerSec); }, 50);
      const begin = performance.now();
      try { await download.start(); } finally { clearInterval(poll); }
      const elapsed = (performance.now() - begin) / 1000;
      assert.equal(download.status, 'Completed', download.errorMessage);
      assert.deepEqual(download.saved, content);
      assert.ok(peakRate > 0);
      results.push({ connections, elapsedSeconds: +elapsed.toFixed(3), measuredMiBps: +(4 / elapsed).toFixed(2), displayedPeakMiBps: +(peakRate / 1024 ** 2).toFixed(2) });
    }
    t.diagnostic(JSON.stringify(results));
  } finally {
    server.closeAllConnections();
    await new Promise(resolve => server.close(resolve));
  }
});

test('IndexedDB handoff waits for commit and rejects transaction aborts', async () => {
  const { saveBlob } = await import('../apps/aurora-extension/core/idb-store.js');
  const previous = globalThis.indexedDB;
  try {
    for (const abort of [false, true]) {
      let request;
      let transaction;
      let closed = false;
      globalThis.indexedDB = { open() {
        const openRequest = {};
        queueMicrotask(() => {
          openRequest.result = {
            transaction() {
              transaction = { objectStore: () => ({ put() { request = {}; return request; } }) };
              return transaction;
            },
            close() { closed = true; }
          };
          openRequest.onsuccess();
        });
        return openRequest;
      } };
      let settled = false;
      const pending = saveBlob('commit-test', new Blob(['hello']));
      pending.then(() => { settled = true; }, () => { settled = true; });
      await new Promise(resolve => setImmediate(resolve));
      request.onsuccess?.();
      await Promise.resolve();
      assert.equal(settled, false, 'request success is not transaction commit');
      if (abort) {
        transaction.error = new Error('Quota exceeded');
        transaction.onabort();
        await assert.rejects(pending, /Quota exceeded/);
      } else {
        transaction.oncomplete();
        assert.equal(await pending, true);
      }
      assert.equal(closed, true);
    }
  } finally {
    if (previous === undefined) delete globalThis.indexedDB;
    else globalThis.indexedDB = previous;
  }
});

test('JavaScript fallback still uses parallel ranges when WASM cannot load', async () => {
  const { WasmDownloadTask: FallbackTask } = await import('../apps/aurora-extension/core/downloader.js?without-wasm');
  const server = installServer();
  const serverFetch = globalThis.fetch;
  globalThis.fetch = (url, options) => String(url).endsWith('.wasm')
    ? Promise.resolve(new Response(null, { status: 503 })) : serverFetch(url, options);
  const download = new FallbackTask('fallback', 'https://example.test/file.bin', 'file.bin', { connections: 4 });
  download.triggerChromeDownload = async blob => { download.saved = new Uint8Array(await blob.arrayBuffer()); download.status = 'Completed'; };
  await download.start();
  assert.equal(download.status, 'Completed', download.errorMessage);
  assert.equal(download.wasmEngine, null);
  assert.deepEqual(download.saved, payload);
  assert.equal(server.requests.filter(r => r.range).length, 4);
});

test('timing rows come from the Rust engine, not JS estimates', async () => {
  installServer();
  const t = task({ connections: 4 });
  await t.start();
  assert.equal(t.status, 'Completed', t.errorMessage);
  const snap = t.getSnapshot();
  assert.ok(typeof snap.startedAtMs === 'number' && snap.startedAtMs > 0);
  assert.ok(typeof snap.endedAtMs === 'number' && snap.endedAtMs >= snap.startedAtMs);
  assert.equal(snap.expectedEndMs, snap.endedAtMs);
  assert.equal(snap.remainingSeconds, null);
  assert.ok(snap.totalTimeSeconds != null && snap.totalTimeSeconds >= 0);
  assert.ok(snap.elapsedSeconds >= snap.totalTimeSeconds);
  assert.ok(snap.averageSpeedBps > 0);
});

test('internal blob URL registration preserves exact custom filename and extension', async () => {
  const { setInternalUrlRegistrar } = await import('../apps/aurora-extension/core/downloader.js');
  let registeredUrl = null;
  let registeredFilename = null;
  setInternalUrlRegistrar((url, filename) => {
    registeredUrl = url;
    registeredFilename = filename;
  });

  const t = new WasmDownloadTask('test_fn', 'https://example.test/500MB-CZIPtestfile.org.zip', '500MB-CZIPtestfile.org.zip');
  globalThis.URL = {
    createObjectURL: (b) => 'blob:chrome-extension://aurora-id/43e399fd-cfff-424d-89f4-26c37bcd9421',
    revokeObjectURL: () => {}
  };
  globalThis.chrome = {
    runtime: {
      sendMessage: async (msg) => ({ success: true })
    },
    downloads: {
      download: async (opts) => 1234,
      onChanged: { addListener: () => {}, removeListener: () => {} },
      search: async () => [{ state: 'complete' }]
    }
  };

  const fakeBlob = new Blob(['test']);
  await t.triggerChromeDownload(fakeBlob);

  assert.equal(registeredUrl, 'blob:chrome-extension://aurora-id/43e399fd-cfff-424d-89f4-26c37bcd9421');
  assert.equal(registeredFilename, '500MB-CZIPtestfile.org.zip');
  assert.equal(t.finalBlobUrl, 'blob:chrome-extension://aurora-id/43e399fd-cfff-424d-89f4-26c37bcd9421');
});
