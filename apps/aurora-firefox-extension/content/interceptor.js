/**
 * AURORA Kaushal IDM - High-Priority Capture Content Script
 * Intercepts download clicks before other extensions or browser default handlers.
 */

// Regex for direct downloadable file extensions
const FILE_EXTENSIONS_REGEX = /\.(zip|rar|7z|tar|gz|bz2|xz|tgz|zst|lz4|iso|img|bin|exe|msi|dmg|pkg|deb|rpm|apk|aab|appimage|jar|whl|crx|wasm|mp4|mkv|avi|mov|wmv|flv|webm|3gp|mp3|flac|wav|aac|m4a|ogg|opus|pdf|doc|docx|xls|xlsx|ppt|pptx|epub|mobi|csv|sqlite|db|sql|vmdk|torrent)(?:[?#]|$)/i;

let autoCaptureEnabled = true;

async function loadSettings() {
  try {
    const { settings } = await chrome.storage.local.get('settings');
    if (settings && typeof settings.autoCapture === 'boolean') {
      autoCaptureEnabled = settings.autoCapture;
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
 * Intercepts direct file links and <a download> clicks with Shadow DOM & modifier key support
 */
window.addEventListener('click', (event) => {
  if (!autoCaptureEnabled) return;

  // Do not intercept non-primary clicks (middle click, right click) or clicks with modifier keys (Ctrl/Cmd/Shift/Alt)
  if (event.button !== 0 || event.ctrlKey || event.metaKey || event.shiftKey || event.altKey) {
    return;
  }

  // Find nearest anchor, supporting Shadow DOM via event.composedPath()
  let anchor = null;
  if (typeof event.composedPath === 'function') {
    const path = event.composedPath();
    for (const el of path) {
      if (el && el.tagName === 'A' && el.href) {
        anchor = el;
        break;
      }
    }
  }
  if (!anchor && event.target && typeof event.target.closest === 'function') {
    anchor = event.target.closest('a');
  }

  if (!anchor || !anchor.href) return;

  const href = anchor.href.trim();
  if (!href || href.startsWith('javascript:') || href.startsWith('#') || href.startsWith('blob:') || href.startsWith('data:') || href.startsWith('mailto:') || href.startsWith('tel:')) {
    return;
  }

  // Intercept if:
  // 1. Has explicit <a download> attribute
  // 2. Links to a file with a downloadable file extension
  const hasDownloadAttr = anchor.hasAttribute('download');
  const isDirectFileLink = FILE_EXTENSIONS_REGEX.test(href);

  if (hasDownloadAttr || isDirectFileLink) {
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
    const segs = url.pathname.split('/').filter(Boolean);
    if (segs.length > 0) {
      const name = decodeURIComponent(segs[segs.length - 1]);
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

  // 1. Create flying file element
  const flyingEl = document.createElement('div');
  flyingEl.className = 'aurora-flying-file';
  flyingEl.style.left = `${startX}px`;
  flyingEl.style.top = `${startY}px`;
  flyingEl.style.setProperty('--delta-x', `${deltaX}px`);
  flyingEl.style.setProperty('--delta-y', `${deltaY}px`);

  flyingEl.innerHTML = `
    <div class="flying-file-card">
      <div class="flying-file-doc">
        <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="#007AFF" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
          <polyline points="14 2 14 8 20 8"></polyline>
          <line x1="16" y1="13" x2="8" y2="13"></line>
          <line x1="16" y1="17" x2="8" y2="17"></line>
        </svg>
      </div>
      <div class="flying-file-badge">${escapeHtml(ext)}</div>
    </div>
  `;

  document.body.appendChild(flyingEl);

  // 2. Remove flying file and open top-right download panel after flight
  setTimeout(() => {
    flyingEl.remove();
    showTopRightDownloadPanel(filename, ext);
  }, 650);
}

function showTopRightDownloadPanel(filename, ext) {
  const existing = document.getElementById('aurora-top-panel-container');
  if (existing) existing.remove();

  const container = document.createElement('div');
  container.id = 'aurora-top-panel-container';
  container.innerHTML = `
    <div class="aurora-top-panel">
      <!-- Target Catch Bucket Header -->
      <div class="aurora-panel-top">
        <div class="aurora-panel-brand">
          <div class="aurora-bucket-landing">
            <svg viewBox="0 0 32 32" width="22" height="22">
              <path class="bucket-tray" d="M6 18 L6 25 C6 26.5 7 27.5 8.5 27.5 L23.5 27.5 C25 27.5 26 26.5 26 25 L26 18" stroke="#007AFF" stroke-width="2.5" stroke-linecap="round" fill="none" />
              <polyline points="10,13 16,19 22,13" fill="none" stroke="#007AFF" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" />
              <line x1="16" y1="5" x2="16" y2="19" stroke="#007AFF" stroke-width="2.5" stroke-linecap="round" />
            </svg>
          </div>
          <span class="aurora-panel-title">Aurora Kaushal Download Manager - Nepal</span>
        </div>
        <div class="aurora-panel-actions">
          <span class="aurora-status-pill">Active</span>
          <button class="aurora-btn-close" id="btn-close-aurora-panel">&times;</button>
        </div>
      </div>

      <!-- File Details -->
      <div class="aurora-panel-body">
        <div class="aurora-panel-file-row">
          <span class="aurora-ext-badge">${escapeHtml(ext)}</span>
          <span class="aurora-filename-text" title="${escapeHtml(filename)}">${escapeHtml(filename)}</span>
        </div>
        <div class="aurora-panel-progress-wrap">
          <div class="aurora-panel-progress-bar"></div>
        </div>
        <div class="aurora-panel-meta">
          <span class="aurora-meta-msg">Accelerated Multi-Streams Downloading...</span>
        </div>
      </div>
    </div>
  `;

  document.body.appendChild(container);

  const btnClose = container.querySelector('#btn-close-aurora-panel');
  if (btnClose) {
    btnClose.addEventListener('click', () => {
      container.classList.add('aurora-panel-fadeout');
      setTimeout(() => container.remove(), 300);
    });
  }

  // Auto dismiss after 5s
  setTimeout(() => {
    if (document.body.contains(container)) {
      container.classList.add('aurora-panel-fadeout');
      setTimeout(() => container.remove(), 350);
    }
  }, 5000);
}

function escapeHtml(str) {
  const div = document.createElement('div');
  div.textContent = str || '';
  return div.innerHTML;
}
