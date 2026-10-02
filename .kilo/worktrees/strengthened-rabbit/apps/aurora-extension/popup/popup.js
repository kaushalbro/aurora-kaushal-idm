/**
 * AURORA Kaushal IDM - Popup Controller & Visualizer
 */

const downloadsContainer = document.getElementById('downloads-container');
const emptyState = document.getElementById('empty-state');
const statTotalSpeed = document.getElementById('stat-total-speed');
const statActiveTasks = document.getElementById('stat-active-tasks');

const addModal = document.getElementById('add-modal');
const btnAddModal = document.getElementById('btn-add-modal');
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

// Initialize
document.addEventListener('DOMContentLoaded', () => {
  refreshDownloads();
  setInterval(refreshDownloads, 500);

  // Setup event listeners
  btnAddModal.addEventListener('click', openAddModal);
  btnCloseModal.addEventListener('click', closeAddModal);
  btnCancelModal.addEventListener('click', closeAddModal);
  btnSubmitDownload.addEventListener('click', submitDownload);
  btnClear.addEventListener('click', clearCompleted);
  btnOptions.addEventListener('click', () => chrome.runtime.openOptionsPage());
});

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
  let totalSpeed = 0;
  let activeTasks = 0;

  downloads.forEach(d => {
    if (d.status === 'Downloading' || d.status === 'Probing') {
      totalSpeed += (d.speedBytesPerSec || 0);
      activeTasks++;
    }
  });

  statTotalSpeed.textContent = formatSpeed(totalSpeed);
  statActiveTasks.textContent = String(activeTasks);

  if (downloads.length === 0) {
    emptyState.style.display = 'flex';
    downloadsContainer.querySelectorAll('.download-card').forEach(el => el.remove());
    return;
  }

  emptyState.style.display = 'none';

  // Render or update each card
  const existingCardIds = new Set();

  downloads.forEach(task => {
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
}

function createDownloadCard(task) {
  const card = document.createElement('div');
  card.id = `card-${task.id}`;
  card.className = 'download-card';

  card.innerHTML = `
    <div class="card-top">
      <div class="file-info">
        <div class="filename" title="${task.filename}">${task.filename}</div>
        <div class="url-sub" title="${task.url}">${task.url}</div>
      </div>
      <div class="card-actions">
        <button class="btn-card btn-pause-resume" data-id="${task.id}">⏸</button>
        <button class="btn-card btn-card-danger btn-cancel" data-id="${task.id}">✕</button>
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

  // Attach button event listeners
  const btnPauseResume = card.querySelector('.btn-pause-resume');
  const btnCancel = card.querySelector('.btn-cancel');

  btnPauseResume.addEventListener('click', () => togglePauseResume(task.id, btnPauseResume.dataset.status));
  btnCancel.addEventListener('click', () => cancelDownload(task.id));

  updateDownloadCard(card, task);
  return card;
}

function updateDownloadCard(card, task) {
  const progressBar = card.querySelector('.progress-bar');
  const metaSize = card.querySelector('.meta-size');
  const metaSpeed = card.querySelector('.meta-speed');
  const statusBadge = card.querySelector('.status-badge');
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

  // Update action buttons
  btnPauseResume.dataset.status = task.status;
  if (task.status === 'Downloading') {
    btnPauseResume.textContent = '⏸';
    btnPauseResume.title = 'Pause';
    btnPauseResume.style.display = 'inline-block';
  } else if (task.status === 'Paused') {
    btnPauseResume.textContent = '▶';
    btnPauseResume.title = 'Resume';
    btnPauseResume.style.display = 'inline-block';
  } else if (task.status === 'Failed') {
    btnPauseResume.textContent = '🔄';
    btnPauseResume.title = 'Retry';
    btnPauseResume.style.display = 'inline-block';
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
  if (!bytesPerSec || bytesPerSec === 0) return '0.00 MB/s';
  const mbps = bytesPerSec / (1024 * 1024);
  if (mbps >= 1.0) {
    return `${mbps.toFixed(2)} MB/s`;
  }
  const kbps = bytesPerSec / 1024;
  return `${kbps.toFixed(1)} KB/s`;
}

function formatDuration(seconds) {
  if (!seconds || seconds <= 0 || !isFinite(seconds)) return '0s';
  const s = Math.round(seconds);
  if (s < 60) return `${s}s`;
  const m = Math.floor(s / 60);
  const remSec = s % 60;
  if (m < 60) return `${m}m ${remSec}s`;
  const h = Math.floor(m / 60);
  const remMin = m % 60;
  return `${h}h ${remMin}m`;
}
