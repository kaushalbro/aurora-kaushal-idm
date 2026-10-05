/**
 * AURORA Kaushal IDM - High-Priority Capture Content Script
 * Intercepts download clicks before other extensions or browser default handlers.
 */

// Regex for direct downloadable file extensions
// Rebuilt at runtime from user settings (options page) when available.
const DEFAULT_EXTENSIONS = ['zip','rar','7z','tar','gz','bz2','xz','tgz','zst','lz4','iso','img','bin','exe','msi','dmg','pkg','deb','rpm','apk','aab','appimage','jar','whl','crx','wasm','mp4','mkv','avi','mov','wmv','flv','webm','3gp','mp3','flac','wav','aac','m4a','ogg','opus','pdf','doc','docx','xls','xlsx','ppt','pptx','epub','mobi','csv','sqlite','db','sql','vmdk','torrent'];
let FILE_EXTENSIONS_REGEX = buildExtRegex(DEFAULT_EXTENSIONS);

// Regex for dynamic download endpoints & query parameters
const DYNAMIC_DOWNLOAD_REGEX = /(?:[?&/](?:download|export|attachment|invoice|report|stream|get_file|get_avoir|fetch_file)[^/]*|[?&](?:dl|export|format|file)=(?:1|true|download|pdf|zip|bin|csv))/i;

function buildExtRegex(list) {
  const esc = (list || []).map(e => String(e).trim().toLowerCase().replace(/^\./, '').replace(/[.*+?^${}()|[\]\\]/g, '\\$&')).filter(Boolean);
  if (!esc.length) return /\.(zip|rar)(?:[?#]|$)/i;
  return new RegExp('\\.(' + esc.join('|') + ')(?:[?#]|$)', 'i');
}

let autoCaptureEnabled = true;

async function loadSettings() {
  try {
    const { settings } = await chrome.storage.local.get('settings');
    if (settings && typeof settings.autoCapture === 'boolean') {
      autoCaptureEnabled = settings.autoCapture;
    }
    if (settings && Array.isArray(settings.interceptExtensions) && settings.interceptExtensions.length) {
      FILE_EXTENSIONS_REGEX = buildExtRegex(settings.interceptExtensions);
    }
  } catch (_) {}
}

loadSettings();

chrome.storage.onChanged.addListener((changes, area) => {
  if (area === 'local' && changes.settings) {
    loadSettings();
  }
});

/**
 * Capture-phase click listener
 * Intercepts direct file links, dynamic download scripts, and <a download> clicks
 */
window.addEventListener('click', (event) => {
  if (!autoCaptureEnabled) return;

  // Do not intercept non-primary clicks (middle click, right click) or clicks with modifier keys (Ctrl/Cmd/Shift/Alt)
  if (event.button !== 0 || event.ctrlKey || event.metaKey || event.shiftKey || event.altKey) {
    return;
  }

  // Find nearest anchor or downloadable button, supporting Shadow DOM via event.composedPath()
  let anchor = null;
  if (typeof event.composedPath === 'function') {
    const path = event.composedPath();
    for (const el of path) {
      if (el && (el.tagName === 'A' || el.hasAttribute?.('data-download-url')) && (el.href || el.getAttribute?.('data-download-url'))) {
        anchor = el;
        break;
      }
    }
  }
  if (!anchor && event.target && typeof event.target.closest === 'function') {
    anchor = event.target.closest('a') || event.target.closest('[data-download-url]');
  }

  if (!anchor) return;

  const rawHref = anchor.href || anchor.getAttribute('data-download-url') || anchor.getAttribute('data-href');
  if (!rawHref || typeof rawHref !== 'string') return;

  const href = rawHref.trim();
  if (!href || href.startsWith('javascript:') || href.startsWith('#') || href.startsWith('blob:') || href.startsWith('data:') || href.startsWith('mailto:') || href.startsWith('tel:')) {
    return;
  }

  // Intercept if:
  // 1. Has explicit <a download> attribute
  // 2. Links to a file with a downloadable file extension
  // 3. Matches dynamic download URL patterns (e.g. get_avoir_pdf.php, download.php)
  const hasDownloadAttr = anchor.hasAttribute('download');
  const isDirectFileLink = FILE_EXTENSIONS_REGEX.test(href);
  const isDynamicDownload = DYNAMIC_DOWNLOAD_REGEX.test(href);

  if (hasDownloadAttr || isDirectFileLink || isDynamicDownload) {
    event.preventDefault();
    event.stopPropagation();
    event.stopImmediatePropagation();

    const rawDownloadAttr = anchor.getAttribute('download');
    const filename = (rawDownloadAttr && rawDownloadAttr.trim() !== '') ? rawDownloadAttr.trim() : extractFilename(href);
    const startX = event.clientX;
    const startY = event.clientY;

    triggerFlyingFileToTop(filename, startX, startY);

    chrome.runtime.sendMessage({
      action: 'ADD_DOWNLOAD',
      url: href,
      filename: filename
    }).catch((err) => {
      console.warn('[AURORA] Failed to queue download:', err);
    });
  }
}, true); // Capture phase

function extractFilename(urlStr) {
  try {
    const url = new URL(urlStr, window.location.href);
    // Check query params for explicit filename hints
    for (const param of ['filename', 'file', 'name', 'title', 'f']) {
      const val = url.searchParams.get(param);
      if (val && val.includes('.')) return decodeURIComponent(val.trim());
    }
    const segs = url.pathname.split('/').filter(Boolean);
    if (segs.length > 0) {
      const name = decodeURIComponent(segs[segs.length - 1].split('?')[0].split('#')[0]);
      if (name.includes('.')) return name;
      return name;
    }
  } catch (_) {}
  return 'download.bin';
}

// Listen for notifications from background service worker (e.g. dynamic downloads)
chrome.runtime.onMessage.addListener((message) => {
  if (message.action === 'SHOW_DOWNLOAD_TOAST' && message.filename) {
    triggerFlyingFileToTop(message.filename);
  }
});

function getFileExtension(filename) {
  if (!filename) return 'FILE';
  const clean = filename.split('?')[0].split('#')[0];
  const parts = clean.split('.');
  if (parts.length > 1) {
    const ext = parts.pop().toUpperCase();
    if (ext.length <= 8) return ext;
  }
  return 'FILE';
}

function triggerFlyingFileToTop(filename, startX, startY) {
  if (!document.body) return;
  // Default to center if coordinates not provided
  if (typeof startX !== 'number' || typeof startY !== 'number') {
    startX = window.innerWidth / 2;
    startY = window.innerHeight / 2;
  }

  const ext = getFileExtension(filename);
  const targetX = Math.max(window.innerWidth - 60, 40);
  const targetY = 16;
  const deltaX = targetX - startX;
  const deltaY = targetY - startY;

  // 1. Create flying file element safely with DOM APIs
  const flyingEl = document.createElement('div');
  flyingEl.className = 'aurora-flying-file';
  flyingEl.style.left = `${startX}px`;
  flyingEl.style.top = `${startY}px`;
  flyingEl.style.setProperty('--delta-x', `${deltaX}px`);
  flyingEl.style.setProperty('--delta-y', `${deltaY}px`);

  const card = document.createElement('div');
  card.className = 'flying-file-card';

  const doc = document.createElement('div');
  doc.className = 'flying-file-doc';

  const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  svg.setAttribute('viewBox', '0 0 24 24');
  svg.setAttribute('width', '22');
  svg.setAttribute('height', '22');
  svg.setAttribute('fill', 'none');
  svg.setAttribute('stroke', '#007AFF');
  svg.setAttribute('stroke-width', '2');
  svg.setAttribute('stroke-linecap', 'round');
  svg.setAttribute('stroke-linejoin', 'round');

  const p1 = document.createElementNS('http://www.w3.org/2000/svg', 'path');
  p1.setAttribute('d', 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z');
  const p2 = document.createElementNS('http://www.w3.org/2000/svg', 'polyline');
  p2.setAttribute('points', '14 2 14 8 20 8');
  const p3 = document.createElementNS('http://www.w3.org/2000/svg', 'line');
  p3.setAttribute('x1', '16'); p3.setAttribute('y1', '13'); p3.setAttribute('x2', '8'); p3.setAttribute('y2', '13');
  const p4 = document.createElementNS('http://www.w3.org/2000/svg', 'line');
  p4.setAttribute('x1', '16'); p4.setAttribute('y1', '17'); p4.setAttribute('x2', '8'); p4.setAttribute('y2', '17');
  svg.append(p1, p2, p3, p4);
  doc.appendChild(svg);

  const badge = document.createElement('div');
  badge.className = 'flying-file-badge';
  badge.textContent = ext;

  card.append(doc, badge);
  flyingEl.appendChild(card);
  document.body.appendChild(flyingEl);

  // 2. Remove flying file and open top-right download panel after flight
  setTimeout(() => {
    flyingEl.remove();
    showTopRightDownloadPanel(filename, ext);
  }, 650);
}

function showTopRightDownloadPanel(filename, ext) {
  if (!document.body) return;
  const existing = document.getElementById('aurora-top-panel-container');
  if (existing) existing.remove();

  const container = document.createElement('div');
  container.id = 'aurora-top-panel-container';

  const panel = document.createElement('div');
  panel.className = 'aurora-top-panel';

  // Header
  const panelTop = document.createElement('div');
  panelTop.className = 'aurora-panel-top';

  const brand = document.createElement('div');
  brand.className = 'aurora-panel-brand';

  const bucket = document.createElement('div');
  bucket.className = 'aurora-bucket-landing';

  const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  svg.setAttribute('viewBox', '0 0 32 32');
  svg.setAttribute('width', '22');
  svg.setAttribute('height', '22');

  const path1 = document.createElementNS('http://www.w3.org/2000/svg', 'path');
  path1.setAttribute('class', 'bucket-tray');
  path1.setAttribute('d', 'M6 18 L6 25 C6 26.5 7 27.5 8.5 27.5 L23.5 27.5 C25 27.5 26 26.5 26 25 L26 18');
  path1.setAttribute('stroke', '#007AFF');
  path1.setAttribute('stroke-width', '2.5');
  path1.setAttribute('stroke-linecap', 'round');
  path1.setAttribute('fill', 'none');

  const poly = document.createElementNS('http://www.w3.org/2000/svg', 'polyline');
  poly.setAttribute('points', '10,13 16,19 22,13');
  poly.setAttribute('fill', 'none');
  poly.setAttribute('stroke', '#007AFF');
  poly.setAttribute('stroke-width', '2.5');
  poly.setAttribute('stroke-linecap', 'round');
  poly.setAttribute('stroke-linejoin', 'round');

  const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
  line.setAttribute('x1', '16'); line.setAttribute('y1', '5'); line.setAttribute('x2', '16'); line.setAttribute('y2', '19');
  line.setAttribute('stroke', '#007AFF');
  line.setAttribute('stroke-width', '2.5');
  line.setAttribute('stroke-linecap', 'round');

  svg.append(path1, poly, line);
  bucket.appendChild(svg);

  const title = document.createElement('span');
  title.className = 'aurora-panel-title';
  title.textContent = 'Aurora Kaushal Download Manager - Nepal';
  brand.append(bucket, title);

  const actions = document.createElement('div');
  actions.className = 'aurora-panel-actions';

  const pill = document.createElement('span');
  pill.className = 'aurora-status-pill';
  pill.textContent = 'Active';

  const btnClose = document.createElement('button');
  btnClose.className = 'aurora-btn-close';
  btnClose.id = 'btn-close-aurora-panel';
  btnClose.textContent = '×';
  actions.append(pill, btnClose);

  panelTop.append(brand, actions);

  // File Details Body
  const panelBody = document.createElement('div');
  panelBody.className = 'aurora-panel-body';

  const fileRow = document.createElement('div');
  fileRow.className = 'aurora-panel-file-row';

  const extBadge = document.createElement('span');
  extBadge.className = 'aurora-ext-badge';
  extBadge.textContent = ext;

  const fnText = document.createElement('span');
  fnText.className = 'aurora-filename-text';
  fnText.title = filename;
  fnText.textContent = filename;
  fileRow.append(extBadge, fnText);

  const progWrap = document.createElement('div');
  progWrap.className = 'aurora-panel-progress-wrap';
  const progBar = document.createElement('div');
  progBar.className = 'aurora-panel-progress-bar';
  progWrap.appendChild(progBar);

  const meta = document.createElement('div');
  meta.className = 'aurora-panel-meta';
  const metaMsg = document.createElement('span');
  metaMsg.className = 'aurora-meta-msg';
  metaMsg.textContent = 'Accelerated Multi-Streams Downloading...';
  meta.appendChild(metaMsg);

  panelBody.append(fileRow, progWrap, meta);
  panel.append(panelTop, panelBody);
  container.appendChild(panel);
  document.body.appendChild(container);

  btnClose.addEventListener('click', () => {
    container.classList.add('aurora-panel-fadeout');
    setTimeout(() => container.remove(), 300);
  });

  // Auto dismiss after 5s
  setTimeout(() => {
    if (document.body.contains(container)) {
      container.classList.add('aurora-panel-fadeout');
      setTimeout(() => container.remove(), 350);
    }
  }, 5000);
}
