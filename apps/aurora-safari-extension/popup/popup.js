/**
 * AURORA Kaushal IDM - Popup Controller & Visualizer
 */

const downloadsContainer = document.getElementById('downloads-container');
const emptyState = document.getElementById('empty-state');
const statTotalSpeed = document.getElementById('stat-total-speed');
const statActiveTasks = document.getElementById('stat-active-tasks');

const selectionBar = document.getElementById('selection-bar');
const checkSelectAll = document.getElementById('check-select-all');
const selectedCountLabel = document.getElementById('selected-count-label');
const btnDeleteSelectedList = document.getElementById('btn-delete-selected-list');
const btnDeleteSelectedFile = document.getElementById('btn-delete-selected-file');

const addModal = document.getElementById('add-modal');
const btnAddModal = document.getElementById('btn-add-modal');
const btnOpenFolder = document.getElementById('btn-open-folder');
const btnCloseModal = document.getElementById('btn-close-modal');
const btnCancelModal = document.getElementById('btn-cancel-modal');
const btnSubmitDownload = document.getElementById('btn-submit-download');
const btnClear = document.getElementById('btn-clear');
const btnOptions = document.getElementById('btn-options');

const inputUrl = document.getElementById('input-url');
const inputFilename = document.getElementById('input-filename');
const selectConnections = document.getElementById('select-connections');
const selectScheduler = document.getElementById('select-scheduler');
const inputChecksum = document.getElementById('input-checksum');
const selectAlgo = document.getElementById('select-algo');

// Desktop App Bridge elements
const desktopSyncBanner = document.getElementById('desktop-sync-banner');
const desktopStatusDot = document.getElementById('desktop-status-dot');
const desktopStatusText = document.getElementById('desktop-status-text');
const desktopStatusSub = document.getElementById('desktop-status-sub');
const btnDownloadDesktop = document.getElementById('btn-download-desktop');
const btnOpenApp = document.getElementById('btn-open-app');
const btnSyncNow = document.getElementById('btn-sync-now');
const lblForwardToggle = document.getElementById('lbl-forward-toggle');
const checkForwardDesktop = document.getElementById('check-forward-desktop');

// Selection & Sync tracking
const selectedTaskIds = new Set();
let currentDownloadsList = [];
let isDesktopConnected = false;
let desktopSyncedDownloads = [];

// Initialize
document.addEventListener('DOMContentLoaded', () => {
  refreshDownloads();
  checkDesktopBridge();
  setInterval(refreshDownloads, 500);
  setInterval(checkDesktopBridge, 2500);

  // Setup event listeners
  btnAddModal.addEventListener('click', openAddModal);
  if (btnOpenFolder) {
    btnOpenFolder.addEventListener('click', openDownloadsFolder);
  }
  btnCloseModal.addEventListener('click', closeAddModal);
  btnCancelModal.addEventListener('click', closeAddModal);
  btnSubmitDownload.addEventListener('click', submitDownload);
  btnClear.addEventListener('click', () => {
    if (confirm('Clear all completed, cancelled, and failed downloads from list?')) {
      clearCompleted();
    }
  });
  btnOptions.addEventListener('click', () => chrome.runtime.openOptionsPage());

  if (btnOpenApp) {
    btnOpenApp.addEventListener('click', async () => {
      try {
        await fetch('http://127.0.0.1:28282/api/open-app', { method: 'POST' });
      } catch (_) {
        window.location.href = 'aurora://open';
      }
    });
  }

  if (btnSyncNow) {
    btnSyncNow.addEventListener('click', async () => {
      btnSyncNow.disabled = true;
      btnSyncNow.style.opacity = '0.6';
      await syncWithDesktop(true);
      btnSyncNow.disabled = false;
      btnSyncNow.style.opacity = '1';
    });
  }

  // Modal backdrop click to dismiss
  if (addModal) {
    addModal.addEventListener('click', (e) => {
      if (e.target === addModal) {
        closeAddModal();
      }
    });
  }

  // Keyboard accessibility
  window.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && !addModal.classList.contains('hidden')) {
      closeAddModal();
    }
  });

  [inputUrl, inputFilename, inputChecksum].forEach(input => {
    if (input) {
      input.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
          e.preventDefault();
          submitDownload();
        }
      });
    }
  });

  // Multi-select bulk listeners
  if (checkSelectAll) {
    checkSelectAll.addEventListener('change', (e) => {
      if (e.target.checked) {
        currentDownloadsList.forEach(t => selectedTaskIds.add(t.id));
      } else {
        selectedTaskIds.clear();
      }
      updateSelectionUI();
    });
  }

  if (btnDeleteSelectedList) {
    btnDeleteSelectedList.addEventListener('click', () => deleteSelected(false));
  }

  if (btnDeleteSelectedFile) {
    btnDeleteSelectedFile.addEventListener('click', () => deleteSelected(true));
  }
});

async function checkDesktopBridge() {
  try {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 600);
    const res = await fetch('http://127.0.0.1:28282/api/status', {
      method: 'GET',
      signal: controller.signal
    });
    clearTimeout(timeoutId);

    if (res.ok) {
      const data = await res.json();
      if (!isDesktopConnected) {
        isDesktopConnected = true;
        updateDesktopBannerUI(true, data.version || '0.1.0');
        syncWithDesktop(false);
      }
      return;
    }
  } catch (_) {
    // Desktop not running
  }

  if (isDesktopConnected) {
    isDesktopConnected = false;
    updateDesktopBannerUI(false);
  }
}

function updateDesktopBannerUI(connected, version = '0.1.0') {
  if (!desktopSyncBanner) return;
  const bannerIcon = document.getElementById('desktop-banner-icon');
  if (connected) {
    desktopSyncBanner.className = 'desktop-banner banner-connected';
    if (desktopStatusDot) desktopStatusDot.className = 'status-dot dot-connected';
    if (desktopStatusText) desktopStatusText.textContent = `🟢 AURORA Desktop Connected (v${version})`;
    if (desktopStatusSub) desktopStatusSub.textContent = 'Hardware Accelerated Engine (32-Streams Active)';
    if (bannerIcon) bannerIcon.title = `AURORA Desktop Connected (v${version})`;
    if (btnDownloadDesktop) btnDownloadDesktop.classList.add('hidden');
    if (btnOpenApp) btnOpenApp.classList.remove('hidden');
    if (btnSyncNow) btnSyncNow.classList.remove('hidden');
    if (lblForwardToggle) lblForwardToggle.classList.remove('hidden');
  } else {
    desktopSyncBanner.className = 'desktop-banner banner-offline';
    if (desktopStatusDot) desktopStatusDot.className = 'status-dot dot-offline';
    if (desktopStatusText) desktopStatusText.textContent = 'Download AURORA for Desktop';
    if (desktopStatusSub) desktopStatusSub.textContent = 'Linux (.deb), Windows (.exe), Mac (.dmg)';
    if (bannerIcon) bannerIcon.title = 'AURORA Desktop Disconnected';
    if (btnDownloadDesktop) btnDownloadDesktop.classList.remove('hidden');
    if (btnOpenApp) btnOpenApp.classList.add('hidden');
    if (btnSyncNow) btnSyncNow.classList.add('hidden');
    if (lblForwardToggle) lblForwardToggle.classList.add('hidden');
  }
}

async function syncWithDesktop(isManual = false) {
  if (!isDesktopConnected) return;
  try {
    const res = await fetch('http://127.0.0.1:28282/api/history');
    if (res.ok) {
      const data = await res.json();
      if (data.success && Array.isArray(data.downloads)) {
        desktopSyncedDownloads = data.downloads.map(d => ({
          id: d.id || 'desk-' + Math.random().toString(36).substring(2, 9),
          url: d.url,
          filename: d.filename,
          destinationPath: d.destination_path,
          totalBytes: d.file_size,
          downloadedBytes: d.downloaded_bytes,
          status: d.status || 'Completed',
          speed: d.average_speed || 0,
          elapsedSeconds: d.elapsed_seconds || 0,
          checksum: d.checksum,
          origin: 'desktop'
        }));
        refreshDownloads();
      }
    }

    // Push local browser downloads to desktop SQLite
    if (currentDownloadsList.length > 0) {
      const recordsToPush = currentDownloadsList
        .filter(d => d.origin !== 'desktop' && d.status === 'Completed')
        .map(d => ({
          id: String(d.id),
          url: d.url,
          filename: d.filename,
          destination_path: d.destinationPath || d.filename,
          file_size: d.totalBytes || null,
          downloaded_bytes: d.downloadedBytes || 0,
          status: d.status,
          elapsed_seconds: d.elapsedSeconds || 0,
          average_speed: d.speed || 0,
          checksum: d.checksum || null,
          created_at: new Date(d.createdAt || Date.now()).toISOString(),
          completed_at: new Date(d.completedAt || Date.now()).toISOString()
        }));

      if (recordsToPush.length > 0) {
        await fetch('http://127.0.0.1:28282/api/history/sync', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ downloads: recordsToPush })
        });
      }
    }
  } catch (err) {
    console.warn('[AURORA] Desktop sync error:', err);
  }
}

async function refreshDownloads() {
  try {
    const response = await chrome.runtime.sendMessage({ action: 'GET_ALL_DOWNLOADS' });
    if (response && response.success) {
      renderDownloads(response.downloads || []);
    }
  } catch (err) {
    // Service worker might be waking up
  }
}

function renderDownloads(downloads) {
  const combinedMap = new Map();
  desktopSyncedDownloads.forEach(d => combinedMap.set(d.url || d.id, d));
  downloads.forEach(d => combinedMap.set(d.url || d.id, d));
  const mergedList = Array.from(combinedMap.values());
  currentDownloadsList = mergedList;

  let totalSpeed = 0;
  let activeTasks = 0;

  mergedList.forEach(d => {
    if (d.status === 'Downloading' || d.status === 'Probing') {
      totalSpeed += (d.speedBytesPerSec || d.speed || 0);
      activeTasks++;
    }
  });

  statTotalSpeed.textContent = formatSpeed(totalSpeed);
  statActiveTasks.textContent = String(activeTasks);

  if (mergedList.length === 0) {
    emptyState.style.display = 'flex';
    if (selectionBar) selectionBar.classList.add('hidden');
    selectedTaskIds.clear();
    downloadsContainer.querySelectorAll('.download-card').forEach(el => el.remove());
    return;
  }

  emptyState.style.display = 'none';
  if (selectionBar) selectionBar.classList.remove('hidden');

  // Prune deleted tasks from selected set
  const currentIds = new Set(mergedList.map(d => d.id));
  for (const id of selectedTaskIds) {
    if (!currentIds.has(id)) {
      selectedTaskIds.delete(id);
    }
  }

  // Render or update each card
  const existingCardIds = new Set();

  mergedList.forEach(task => {
    existingCardIds.add(task.id);
    let card = document.getElementById(`card-${task.id}`);

    if (!card) {
      card = createDownloadCard(task);
      downloadsContainer.appendChild(card);
    } else {
      updateDownloadCard(card, task);
    }
  });

  // Remove stale cards
  downloadsContainer.querySelectorAll('.download-card').forEach(card => {
    const id = card.id.replace('card-', '');
    if (!existingCardIds.has(id)) {
      card.remove();
    }
  });

  updateSelectionUI();
}

function updateSelectionUI() {
  const total = currentDownloadsList.length;
  const count = selectedTaskIds.size;

  if (selectedCountLabel) {
    selectedCountLabel.textContent = `${count} of ${total} selected`;
  }

  if (checkSelectAll) {
    checkSelectAll.checked = count > 0 && count === total;
    checkSelectAll.indeterminate = count > 0 && count < total;
  }

  if (btnDeleteSelectedList) {
    btnDeleteSelectedList.style.opacity = count > 0 ? '1' : '0.45';
    btnDeleteSelectedList.style.pointerEvents = count > 0 ? 'auto' : 'none';
  }

  if (btnDeleteSelectedFile) {
    btnDeleteSelectedFile.style.opacity = count > 0 ? '1' : '0.45';
    btnDeleteSelectedFile.style.pointerEvents = count > 0 ? 'auto' : 'none';
  }

  // Sync checkboxes on cards
  downloadsContainer.querySelectorAll('.card-checkbox').forEach(cb => {
    cb.checked = selectedTaskIds.has(cb.dataset.id);
  });
}

function escapeHtml(str) {
  if (!str) return '';
  return String(str)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

function createDownloadCard(task) {
  const card = document.createElement('div');
  card.id = `card-${task.id}`;
  card.className = 'download-card';

  const safeFilename = escapeHtml(task.filename || 'download');
  const safeUrl = escapeHtml(task.url || '');

  card.innerHTML = `
    <div class="card-top">
      <div class="card-select-wrap">
        <label class="custom-checkbox-wrap" title="Select download">
          <input type="checkbox" class="card-checkbox" data-id="${task.id}">
          <span class="custom-checkbox-box"></span>
        </label>
      </div>
      <div class="file-info">
        <div class="filename" title="${safeFilename}">${safeFilename}</div>
        <div class="url-sub" title="${safeUrl}">${safeUrl}</div>
      </div>
      <div class="card-actions">
        <button class="btn-card btn-copy-url" data-url="${safeUrl}" title="Copy download link">
          <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path>
            <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path>
          </svg>
        </button>
        <button class="btn-card btn-open-file" data-id="${task.id}" data-download-id="${task.downloadId || ''}" title="Show in folder (File Explorer)">
          <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg>
        </button>
        <button class="btn-card btn-pause-resume" data-id="${task.id}" title="Action"></button>
        <button class="btn-card btn-remove-list" data-id="${task.id}" title="Remove from list">
          <svg viewBox="0 0 24 24" width="11" height="11" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
        </button>
        <button class="btn-card btn-card-danger btn-delete-disk" data-id="${task.id}" title="Delete file from disk and list">
          <svg viewBox="0 0 24 24" width="11" height="11" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>
        </button>
      </div>
    </div>

    <div class="progress-container">
      <div class="progress-bar" style="width: ${task.progressPct || 0}%"></div>
    </div>

    <div class="card-meta">
      <span class="meta-size">${formatBytes(task.downloadedBytes)} / ${task.totalBytes ? formatBytes(task.totalBytes) : 'Unknown'} (${(task.progressPct || 0).toFixed(1)}%)</span>
      <span class="meta-speed">${task.status === 'Downloading' ? formatSpeed(task.speedBytesPerSec) : ''}</span>
      <span class="status-badge status-${task.status.toLowerCase()}">${task.status}</span>
    </div>

    <div class="segment-map"></div>
  `;

  // Attach button & checkbox event listeners
  const checkbox = card.querySelector('.card-checkbox');
  const btnCopyUrl = card.querySelector('.btn-copy-url');
  const btnOpenFile = card.querySelector('.btn-open-file');
  const btnPauseResume = card.querySelector('.btn-pause-resume');
  const btnRemoveList = card.querySelector('.btn-remove-list');
  const btnDeleteDisk = card.querySelector('.btn-delete-disk');

  checkbox.addEventListener('change', (e) => {
    if (e.target.checked) {
      selectedTaskIds.add(task.id);
    } else {
      selectedTaskIds.delete(task.id);
    }
    updateSelectionUI();
  });

  btnCopyUrl.addEventListener('click', () => copyTaskUrl(task.url, btnCopyUrl));
  btnOpenFile.addEventListener('click', () => openDownloadedFile(task.id, btnOpenFile.dataset.downloadId));
  btnPauseResume.addEventListener('click', () => togglePauseResume(task.id, btnPauseResume.dataset.status));
  btnRemoveList.addEventListener('click', () => deleteTask(task.id, false));
  btnDeleteDisk.addEventListener('click', () => {
    if (confirm(`Delete "${task.filename}" from disk and list?`)) {
      deleteTask(task.id, true);
    }
  });

  updateDownloadCard(card, task);
  return card;
}

function updateDownloadCard(card, task) {
  const progressBar = card.querySelector('.progress-bar');
  const metaSize = card.querySelector('.meta-size');
  const metaSpeed = card.querySelector('.meta-speed');
  const statusBadge = card.querySelector('.status-badge');
  const btnOpenFile = card.querySelector('.btn-open-file');
  const btnPauseResume = card.querySelector('.btn-pause-resume');
  const segmentMap = card.querySelector('.segment-map');

  // Update progress bar
  progressBar.style.width = `${task.progressPct || 0}%`;
  progressBar.className = `progress-bar ${task.status.toLowerCase()}`;

  // Update text
  metaSize.textContent = `${formatBytes(task.downloadedBytes)} / ${task.totalBytes ? formatBytes(task.totalBytes) : 'Unknown'} (${(task.progressPct || 0).toFixed(1)}%)`;
  
  if (task.status === 'Downloading') {
    const etaText = task.etaSeconds ? ` (ETA: ${formatDuration(task.etaSeconds)})` : '';
    metaSpeed.textContent = `${formatSpeed(task.speedBytesPerSec)}${etaText}`;
  } else if (task.status === 'Failed') {
    metaSpeed.textContent = task.errorMessage ? `Error: ${task.errorMessage}` : 'Download Failed';
  } else {
    metaSpeed.textContent = '';
  }

  statusBadge.textContent = task.status;
  statusBadge.className = `status-badge status-${task.status.toLowerCase()}`;

  // Update folder button download ID
  if (btnOpenFile) {
    btnOpenFile.dataset.downloadId = task.downloadId || '';
  }

  btnPauseResume.dataset.status = task.status;
  if (task.status === 'Downloading') {
    btnPauseResume.innerHTML = '<svg viewBox="0 0 24 24" width="11" height="11" fill="currentColor"><rect x="6" y="4" width="4" height="16" rx="1"></rect><rect x="14" y="4" width="4" height="16" rx="1"></rect></svg>';
    btnPauseResume.title = 'Pause';
    btnPauseResume.style.display = 'inline-flex';
  } else if (task.status === 'Paused') {
    btnPauseResume.innerHTML = '<svg viewBox="0 0 24 24" width="11" height="11" fill="currentColor"><polygon points="5 3 19 12 5 21 5 3"></polygon></svg>';
    btnPauseResume.title = 'Resume';
    btnPauseResume.style.display = 'inline-flex';
  } else if (task.status === 'Failed') {
    btnPauseResume.innerHTML = '<svg viewBox="0 0 24 24" width="11" height="11" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="1 4 1 10 7 10"></polyline><path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10"></path></svg>';
    btnPauseResume.title = 'Retry';
    btnPauseResume.style.display = 'inline-flex';
  } else {
    btnPauseResume.style.display = 'none';
  }

  // Render segment map blocks if multi-segment
  if (task.segments && task.segments.length > 0 && task.totalBytes) {
    segmentMap.style.display = 'flex';
    segmentMap.innerHTML = '';

    task.segments.forEach(seg => {
      const segBlock = document.createElement('div');
      const segLen = seg.end - seg.start + 1;
      const pct = (segLen / task.totalBytes) * 100;
      segBlock.className = `seg-block seg-${seg.state}`;
      segBlock.style.width = `${pct}%`;
      segBlock.title = `Segment #${seg.id}: [${seg.start}-${seg.end}] (${seg.state})`;
      segmentMap.appendChild(segBlock);
    });
  } else {
    segmentMap.style.display = 'none';
  }
}

async function togglePauseResume(taskId, currentStatus) {
  if (currentStatus === 'Downloading') {
    await chrome.runtime.sendMessage({ action: 'PAUSE_DOWNLOAD', taskId });
  } else if (currentStatus === 'Paused') {
    await chrome.runtime.sendMessage({ action: 'RESUME_DOWNLOAD', taskId });
  }
  refreshDownloads();
}

async function cancelDownload(taskId) {
  await chrome.runtime.sendMessage({ action: 'CANCEL_DOWNLOAD', taskId });
  refreshDownloads();
}

async function deleteTask(taskId, deleteFiles = false) {
  try {
    await chrome.runtime.sendMessage({
      action: 'DELETE_TASKS',
      taskIds: [taskId],
      deleteFiles: Boolean(deleteFiles)
    });
    selectedTaskIds.delete(taskId);
    refreshDownloads();
  } catch (err) {
    console.error('[AURORA] Failed to delete task:', err);
  }
}

async function deleteSelected(deleteFiles = false) {
  const ids = Array.from(selectedTaskIds);
  if (ids.length === 0) return;

  const msg = deleteFiles
    ? `Are you sure you want to delete ${ids.length} selected file(s) from disk and list?`
    : `Remove ${ids.length} selected item(s) from download list?`;

  if (confirm(msg)) {
    try {
      await chrome.runtime.sendMessage({
        action: 'DELETE_TASKS',
        taskIds: ids,
        deleteFiles: Boolean(deleteFiles)
      });
      selectedTaskIds.clear();
      refreshDownloads();
    } catch (err) {
      alert('Deletion error: ' + err.message);
    }
  }
}

async function clearCompleted() {
  await chrome.runtime.sendMessage({ action: 'CLEAR_COMPLETED' });
  refreshDownloads();
}

function openAddModal() {
  addModal.classList.remove('hidden');
  inputUrl.value = '';
  inputFilename.value = '';
  inputChecksum.value = '';
  inputUrl.focus();

  // Try reading clipboard URL
  navigator.clipboard.readText().then(text => {
    if (text && (text.startsWith('http://') || text.startsWith('https://'))) {
      inputUrl.value = text.trim();
    }
  }).catch(() => {});
}

function closeAddModal() {
  addModal.classList.add('hidden');
}

async function submitDownload() {
  const url = inputUrl.value.trim();
  if (!url) {
    inputUrl.focus();
    return;
  }

  const filename = inputFilename.value.trim() || null;
  const connections = parseInt(selectConnections.value, 10);
  const schedulerType = selectScheduler.value;
  const checksum = inputChecksum.value.trim() || null;
  const hashAlgo = selectAlgo.value;

  btnSubmitDownload.disabled = true;
  btnSubmitDownload.textContent = 'Starting...';

  // If Desktop App is active and Auto-Route is checked, forward directly to Desktop Engine!
  if (isDesktopConnected && checkForwardDesktop && checkForwardDesktop.checked) {
    try {
      const res = await fetch('http://127.0.0.1:28282/api/download', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          url,
          filename,
          connections
        })
      });
      if (res.ok) {
        closeAddModal();
        await syncWithDesktop(true);
        btnSubmitDownload.disabled = false;
        btnSubmitDownload.textContent = 'Start Download';
        return;
      }
    } catch (e) {
      console.warn('[AURORA] Desktop route fallback to browser:', e);
    }
  }

  try {
    const res = await chrome.runtime.sendMessage({
      action: 'ADD_DOWNLOAD',
      url,
      filename,
      connections,
      schedulerType,
      checksum,
      hashAlgo
    });

    if (res && res.success) {
      closeAddModal();
      refreshDownloads();
    }
  } catch (err) {
    alert('Failed to start download: ' + err.message);
  } finally {
    btnSubmitDownload.disabled = false;
    btnSubmitDownload.textContent = 'Start Download';
  }
}

function formatBytes(bytes) {
  if (!bytes || bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

function formatSpeed(bytesPerSec) {
  if (!bytesPerSec || bytesPerSec === 0 || !isFinite(bytesPerSec)) return '0.00 MB/s';
  const mbps = bytesPerSec / (1024 * 1024);
  if (mbps >= 1.0) {
    return `${mbps.toFixed(2)} MB/s`;
  }
  const kbps = bytesPerSec / 1024;
  if (kbps >= 1.0) {
    return `${kbps.toFixed(1)} KB/s`;
  }
  return `${Math.round(bytesPerSec)} B/s`;
}

function formatDuration(seconds) {
  if (!seconds || seconds <= 0 || !isFinite(seconds)) return '--';
  const s = Math.round(seconds);
  if (s < 60) return `${s}s`;
  const m = Math.floor(s / 60);
  const remSec = s % 60;
  if (m < 60) return `${m}m ${remSec}s`;
  const h = Math.floor(m / 60);
  const remMin = m % 60;
  return `${h}h ${remMin}m`;
}

function openDownloadsFolder() {
  if (typeof chrome !== 'undefined' && chrome.downloads?.showDefaultFolder) {
    chrome.downloads.showDefaultFolder();
  }
}

function openDownloadedFile(taskId, downloadIdStr) {
  const downloadId = parseInt(downloadIdStr, 10);
  if (downloadId && !isNaN(downloadId) && typeof chrome !== 'undefined' && chrome.downloads?.show) {
    try {
      chrome.downloads.show(downloadId);
      return;
    } catch (e) {
      console.warn('[AURORA] Could not show specific download:', e);
    }
  }
  // Fallback to opening default downloads folder
  if (typeof chrome !== 'undefined' && chrome.downloads?.showDefaultFolder) {
    chrome.downloads.showDefaultFolder();
  }
}

async function copyTaskUrl(url, btnEl) {
  if (!url) return;
  try {
    await navigator.clipboard.writeText(url);
    if (btnEl) {
      const originalHtml = btnEl.innerHTML;
      const originalTitle = btnEl.title;
      btnEl.innerHTML = `<svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="#34c759" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>`;
      btnEl.title = 'Link copied!';
      setTimeout(() => {
        btnEl.innerHTML = originalHtml;
        btnEl.title = originalTitle;
      }, 1500);
    }
  } catch (err) {
    console.error('[AURORA] Failed to copy URL via clipboard API, trying execCommand fallback:', err);
    try {
      const textarea = document.createElement('textarea');
      textarea.value = url;
      textarea.style.position = 'fixed';
      textarea.style.opacity = '0';
      document.body.appendChild(textarea);
      textarea.focus();
      textarea.select();
      document.execCommand('copy');
      document.body.removeChild(textarea);
      if (btnEl) {
        const originalHtml = btnEl.innerHTML;
        const originalTitle = btnEl.title;
        btnEl.innerHTML = `<svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="#34c759" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>`;
        btnEl.title = 'Link copied!';
        setTimeout(() => {
          btnEl.innerHTML = originalHtml;
          btnEl.title = originalTitle;
        }, 1500);
      }
    } catch (e) {
      console.error('[AURORA] Clipboard copy fallback failed:', e);
    }
  }
}


