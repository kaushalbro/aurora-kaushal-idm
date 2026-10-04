import { formatSpeed, formatNetworkSpeed } from '../core/format.js';

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
  const withTimeout = (ms) => {
    const ctrl = new AbortController();
    const t = setTimeout(() => ctrl.abort(), ms);
    return { signal: ctrl.signal, done: () => clearTimeout(t) };
  };
  try {
    const h = withTimeout(4000);
    let res;
    try {
      res = await fetch('http://127.0.0.1:28282/api/history', { signal: h.signal });
    } finally { h.done(); }
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
        const p = withTimeout(4000);
        try {
          await fetch('http://127.0.0.1:28282/api/history/sync', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ downloads: recordsToPush }),
            signal: p.signal
          });
        } finally { p.done(); }
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
  // Key by stable task id. Keying by URL collapses re-downloads of the same
  // file and desktop/browser duplicates into one row (history loss).
  const combinedMap = new Map();
  desktopSyncedDownloads.forEach(d => combinedMap.set(d.id, d));
  downloads.forEach(d => combinedMap.set(d.id, d));
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
  statTotalSpeed.title = `${formatNetworkSpeed(totalSpeed)} — 8 bits = 1 byte`;
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

function createSvg(viewBox, width, height, elements, extraAttrs = {}) {
  const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  svg.setAttribute('viewBox', viewBox);
  svg.setAttribute('width', String(width));
  svg.setAttribute('height', String(height));
  for (const [k, v] of Object.entries(extraAttrs)) {
    svg.setAttribute(k, v);
  }
  for (const el of elements) {
    const node = document.createElementNS('http://www.w3.org/2000/svg', el.tag);
    for (const [k, v] of Object.entries(el.attrs || {})) {
      node.setAttribute(k, v);
    }
    svg.appendChild(node);
  }
  return svg;
}

function createDownloadCard(task) {
  const card = document.createElement('div');
  card.id = `card-${task.id}`;
  card.className = 'download-card';

  // Card Top
  const cardTop = document.createElement('div');
  cardTop.className = 'card-top';

  // Checkbox wrap
  const selectWrap = document.createElement('div');
  selectWrap.className = 'card-select-wrap';
  const labelWrap = document.createElement('label');
  labelWrap.className = 'custom-checkbox-wrap';
  labelWrap.title = 'Select download';
  const checkbox = document.createElement('input');
  checkbox.type = 'checkbox';
  checkbox.className = 'card-checkbox';
  checkbox.dataset.id = task.id;
  const checkboxBox = document.createElement('span');
  checkboxBox.className = 'custom-checkbox-box';
  labelWrap.append(checkbox, checkboxBox);
  selectWrap.appendChild(labelWrap);

  // File info
  const fileInfo = document.createElement('div');
  fileInfo.className = 'file-info';
  const fnDiv = document.createElement('div');
  fnDiv.className = 'filename';
  fnDiv.title = task.filename || 'download';
  fnDiv.textContent = task.filename || 'download';
  const urlSub = document.createElement('div');
  urlSub.className = 'url-sub';
  urlSub.title = task.url || '';
  urlSub.textContent = task.url || '';
  fileInfo.append(fnDiv, urlSub);

  // Card actions
  const cardActions = document.createElement('div');
  cardActions.className = 'card-actions';

  const btnCopyUrl = document.createElement('button');
  btnCopyUrl.className = 'btn-card btn-copy-url';
  btnCopyUrl.dataset.url = task.url || '';
  btnCopyUrl.title = 'Copy download link';
  btnCopyUrl.appendChild(createSvg('0 0 24 24', 12, 12, [
    { tag: 'path', attrs: { d: 'M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71' } },
    { tag: 'path', attrs: { d: 'M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71' } }
  ], { fill: 'none', stroke: 'currentColor', 'stroke-width': '2', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }));

  const btnOpenFile = document.createElement('button');
  btnOpenFile.className = 'btn-card btn-open-file';
  btnOpenFile.dataset.id = task.id;
  btnOpenFile.dataset.downloadId = task.downloadId || '';
  btnOpenFile.title = 'Show in folder (File Explorer)';
  btnOpenFile.appendChild(createSvg('0 0 24 24', 12, 12, [
    { tag: 'path', attrs: { d: 'M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z' } }
  ], { fill: 'none', stroke: 'currentColor', 'stroke-width': '2', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }));

  const btnPauseResume = document.createElement('button');
  btnPauseResume.className = 'btn-card btn-pause-resume';
  btnPauseResume.dataset.id = task.id;
  btnPauseResume.title = 'Action';

  const btnRemoveList = document.createElement('button');
  btnRemoveList.className = 'btn-card btn-remove-list';
  btnRemoveList.dataset.id = task.id;
  btnRemoveList.title = 'Remove from list';
  btnRemoveList.appendChild(createSvg('0 0 24 24', 11, 11, [
    { tag: 'line', attrs: { x1: '18', y1: '6', x2: '6', y2: '18' } },
    { tag: 'line', attrs: { x1: '6', y1: '6', x2: '18', y2: '18' } }
  ], { fill: 'none', stroke: 'currentColor', 'stroke-width': '2.5', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }));

  const btnDeleteDisk = document.createElement('button');
  btnDeleteDisk.className = 'btn-card btn-card-danger btn-delete-disk';
  btnDeleteDisk.dataset.id = task.id;
  btnDeleteDisk.title = 'Delete file from disk and list';
  btnDeleteDisk.appendChild(createSvg('0 0 24 24', 11, 11, [
    { tag: 'polyline', attrs: { points: '3 6 5 6 21 6' } },
    { tag: 'path', attrs: { d: 'M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2' } }
  ], { fill: 'none', stroke: 'currentColor', 'stroke-width': '2', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }));

  cardActions.append(btnCopyUrl, btnOpenFile, btnPauseResume, btnRemoveList, btnDeleteDisk);
  cardTop.append(selectWrap, fileInfo, cardActions);

  // Progress Bar Container
  const progContainer = document.createElement('div');
  progContainer.className = 'progress-container';
  const progBar = document.createElement('div');
  progBar.className = 'progress-bar';
  progBar.style.width = `${task.progressPct || 0}%`;
  progContainer.appendChild(progBar);

  // Card Meta
  const cardMeta = document.createElement('div');
  cardMeta.className = 'card-meta';
  const metaSize = document.createElement('span');
  metaSize.className = 'meta-size';
  const metaSpeed = document.createElement('span');
  metaSpeed.className = 'meta-speed';
  const statusBadge = document.createElement('span');
  statusBadge.className = `status-badge status-${task.status.toLowerCase()}`;
  statusBadge.textContent = task.status;
  cardMeta.append(metaSize, metaSpeed, statusBadge);

  // Segment Map
  const segmentMap = document.createElement('div');
  segmentMap.className = 'segment-map';

  card.append(cardTop, progContainer, cardMeta, segmentMap);

  // Attach button & checkbox event listeners
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
    metaSpeed.title = `${formatNetworkSpeed(task.speedBytesPerSec)} — 8 bits = 1 byte`;
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
    btnPauseResume.replaceChildren(createSvg('0 0 24 24', 11, 11, [
      { tag: 'rect', attrs: { x: '6', y: '4', width: '4', height: '16', rx: '1' } },
      { tag: 'rect', attrs: { x: '14', y: '4', width: '4', height: '16', rx: '1' } }
    ], { fill: 'currentColor' }));
    btnPauseResume.title = 'Pause';
    btnPauseResume.style.display = 'inline-flex';
  } else if (task.status === 'Paused') {
    btnPauseResume.replaceChildren(createSvg('0 0 24 24', 11, 11, [
      { tag: 'polygon', attrs: { points: '5 3 19 12 5 21 5 3' } }
    ], { fill: 'currentColor' }));
    btnPauseResume.title = 'Resume';
    btnPauseResume.style.display = 'inline-flex';
  } else if (task.status === 'Failed') {
    btnPauseResume.replaceChildren(createSvg('0 0 24 24', 11, 11, [
      { tag: 'polyline', attrs: { points: '1 4 1 10 7 10' } },
      { tag: 'path', attrs: { d: 'M3.51 15a9 9 0 1 0 2.13-9.36L1 10' } }
    ], { fill: 'none', stroke: 'currentColor', 'stroke-width': '2.5', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }));
    btnPauseResume.title = 'Retry';
    btnPauseResume.style.display = 'inline-flex';
  } else {
    btnPauseResume.style.display = 'none';
  }

  // Render segment map blocks if multi-segment
  if (task.segments && task.segments.length > 0 && task.totalBytes) {
    segmentMap.style.display = 'flex';
    segmentMap.replaceChildren();

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
  if (currentStatus === 'Downloading' || currentStatus === 'Probing' || currentStatus === 'Queued') {
    await chrome.runtime.sendMessage({ action: 'PAUSE_DOWNLOAD', taskId });
  } else if (currentStatus === 'Paused' || currentStatus === 'Failed') {
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

function showCopiedIndicator(btnEl) {
  if (!btnEl) return;
  const originalTitle = btnEl.title;
  const checkSvg = createSvg('0 0 24 24', 12, 12, [
    { tag: 'polyline', attrs: { points: '20 6 9 17 4 12' } }
  ], { fill: 'none', stroke: '#34c759', 'stroke-width': '2.5', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' });
  
  const copySvg = createSvg('0 0 24 24', 12, 12, [
    { tag: 'path', attrs: { d: 'M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71' } },
    { tag: 'path', attrs: { d: 'M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71' } }
  ], { fill: 'none', stroke: 'currentColor', 'stroke-width': '2', 'stroke-linecap': 'round', 'stroke-linejoin': 'round' });

  btnEl.replaceChildren(checkSvg);
  btnEl.title = 'Link copied!';
  setTimeout(() => {
    btnEl.replaceChildren(copySvg);
    btnEl.title = originalTitle;
  }, 1500);
}

async function copyTaskUrl(url, btnEl) {
  if (!url) return;
  try {
    await navigator.clipboard.writeText(url);
    showCopiedIndicator(btnEl);
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
      showCopiedIndicator(btnEl);
    } catch (e) {
      console.error('[AURORA] Clipboard copy fallback failed:', e);
    }
  }
}


