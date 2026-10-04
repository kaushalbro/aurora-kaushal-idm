/**
 * Aurora Kaushal Download Manager - Nepal
 * Service Worker (Manifest V3) - Intelligent High-Performance Interceptor
 */
import { WasmDownloadTask, ensureWasmLoaded } from '../core/downloader.js';

// Active download tasks in memory
const tasks = new Map();

// Set to track downloads initiated by AURORA itself so we don't intercept our own saves
const internalDownloadUrls = new Set();

// Default Settings
const DEFAULT_SETTINGS = {
  autoCapture: true,
  connections: 8,
  schedulerType: 'aurora-ect',
  minFileSizeMb: 0,
  interceptAllDownloads: true
};

// Storage Key for Persisted Task History
const STORAGE_KEY_TASKS = 'aurora_persisted_tasks';

/**
 * Task Snapshot Stub for rehydrated items loaded from storage
 */
class PersistedTaskStub {
  constructor(snapshot) {
    this.id = snapshot.id;
    this.downloadId = snapshot.downloadId || null;
    this.filename = snapshot.filename || 'download.bin';
    this.url = snapshot.url || '';
    this.options = Object.fromEntries(['connections', 'schedulerType', 'checksum', 'hashAlgo']
      .filter(key => snapshot[key] !== undefined).map(key => [key, snapshot[key]]));
    this.restarting = false;
    this.totalBytes = snapshot.totalBytes || null;
    this.downloadedBytes = snapshot.downloadedBytes || 0;
    this.status = snapshot.status || 'Completed';
    this.errorMessage = snapshot.errorMessage || null;
    this.speedBytesPerSec = 0;
    this.speedMbps = 0;
    this.etaSeconds = null;
    this.progressPct = snapshot.progressPct || (this.totalBytes ? (this.downloadedBytes / this.totalBytes) * 100 : 100);
    this.segments = snapshot.segments || [];
    this.activeConnections = 0;
  }

  getSnapshot() {
    return {
      id: this.id,
      downloadId: this.downloadId,
      ...this.options,
      filename: this.filename,
      url: this.url,
      totalBytes: this.totalBytes,
      downloadedBytes: this.downloadedBytes,
      status: this.status,
      errorMessage: this.errorMessage,
      speedBytesPerSec: this.speedBytesPerSec,
      speedMbps: this.speedMbps,
      etaSeconds: this.etaSeconds,
      progressPct: this.progressPct,
      segments: this.segments,
      activeConnections: 0
    };
  }

  pause() { this.status = 'Paused'; }
  async resume() {
    if (this.restarting || !['Paused', 'Failed'].includes(this.status)) return;
    this.restarting = true;
    try {
      const { settings = DEFAULT_SETTINGS } = await chrome.storage.local.get('settings');
      if (this.status === 'Cancelled') return;
      const id = await createDownloadTask(this.url, this.filename, { ...settings, ...this.options });
      tasks.delete(this.id);
      await persistTasks();
      return id;
    } finally {
      this.restarting = false;
    }
  }
  cancel() { this.status = 'Cancelled'; }
}

/**
 * Synchronize task snapshots into persistent storage
 */
async function persistTasks() {
  try {
    const list = Array.from(tasks.values()).map(t => typeof t.getSnapshot === 'function' ? t.getSnapshot() : t);
    const trimmed = list.slice(-200);
    await chrome.storage.local.set({ [STORAGE_KEY_TASKS]: trimmed });
  } catch (err) {
    console.warn('[AURORA] Failed to persist tasks:', err);
  }
}

/**
 * Rehydrate tasks from persistent storage on Service Worker startup/wake
 */
async function rehydrateTasks() {
  try {
    const data = await chrome.storage.local.get(STORAGE_KEY_TASKS);
    if (data && Array.isArray(data[STORAGE_KEY_TASKS])) {
      for (const snap of data[STORAGE_KEY_TASKS]) {
        if (snap && snap.id && !tasks.has(snap.id)) {
          if (snap.status === 'Downloading' || snap.status === 'Probing' || snap.status === 'Assembling' || snap.status === 'Verifying' || snap.status === 'Queued') {
            snap.status = 'Paused';
          }
          tasks.set(snap.id, new PersistedTaskStub(snap));
        }
      }
    }
  } catch (err) {
    console.warn('[AURORA] Failed to rehydrate tasks:', err);
  }
  updateBadge();
}

// Keep-alive alarm management to prevent Service Worker sleep during active downloads
function checkKeepAliveAlarm() {
  const hasActive = Array.from(tasks.values()).some(t =>
    t.status === 'Downloading' || t.status === 'Probing' || t.status === 'Assembling' || t.status === 'Verifying'
  );

  if (hasActive) {
    chrome.alarms.create('aurora-keepalive', { periodInMinutes: 0.5 });
  } else {
    chrome.alarms.clear('aurora-keepalive').catch(() => {});
  }
}

chrome.alarms.onAlarm.addListener((alarm) => {
  if (alarm.name === 'aurora-keepalive') {
    updateBadge();
    checkKeepAliveAlarm();
    persistTasks();
  }
});

// Initialize context menus and settings on install
chrome.runtime.onInstalled.addListener(async () => {
  const current = await chrome.storage.local.get('settings');
  if (!current.settings) {
    await chrome.storage.local.set({ settings: DEFAULT_SETTINGS });
  }

  // Remove existing context menus first to avoid duplicate ID errors
  chrome.contextMenus.removeAll(() => {
    chrome.contextMenus.create({
      id: 'aurora-download-link',
      title: 'Download with Aurora Kaushal IDM',
      contexts: ['link', 'image', 'video', 'audio', 'selection']
    }, () => {
      if (chrome.runtime.lastError) {
        // Ignored
      }
    });
  });

  console.log('[AURORA] Service worker registered and context menus created.');
});

// Handle Context Menu clicks
chrome.contextMenus.onClicked.addListener(async (info) => {
  const targetUrl = (info.linkUrl || info.srcUrl || info.selectionText || '').trim();
  if (!targetUrl) return;

  const { settings = DEFAULT_SETTINGS } = await chrome.storage.local.get('settings');
  await createDownloadTask(targetUrl, null, settings);
});

// Rehydrate on browser startup or worker wake-up
chrome.runtime.onStartup.addListener(() => {
  rehydrateTasks();
});

rehydrateTasks();

/**
 * Universal Download Interception:
 * Whenever browser begins ANY download, AURORA intercepts it and accelerates with WASM.
 */
chrome.downloads.onCreated.addListener(async (downloadItem) => {
  const { settings = DEFAULT_SETTINGS } = await chrome.storage.local.get('settings');
  if (settings.autoCapture === false) return;

  const url = downloadItem.finalUrl || downloadItem.url;
  if (!url) return;

  // Ignore internal blob/data URLs or downloads triggered by AURORA itself
  if (url.startsWith('blob:') || url.startsWith('data:') || url.startsWith('chrome:') || url.startsWith('moz-extension:') || internalDownloadUrls.has(url)) {
    return;
  }

  // Check MIME type: don't intercept accidental HTML "Save Page As" unless requested
  const mime = (downloadItem.mime || '').toLowerCase();
  if (mime === 'text/html' && !url.toLowerCase().includes('download') && !downloadItem.filename.includes('.')) {
    return;
  }

  console.log(`[AURORA] ⚡ Intercepted download: ${url} (MIME: ${mime}, Filename: ${downloadItem.filename})`);

  // 1. Cancel browser's slow native download immediately
  try {
    await chrome.downloads.cancel(downloadItem.id);
    await chrome.downloads.erase({ id: downloadItem.id });
  } catch (e) {
    console.warn('[AURORA] Could not cancel native download:', e);
  }

  // 2. Extract suggested filename if browser already knew it
  const suggestedFilename = downloadItem.filename ? downloadItem.filename.split(/[\/\\]/).pop() : null;

  // 3. Hand over to AURORA's high-speed WebAssembly Multi-Stream Downloader
  await createDownloadTask(url, suggestedFilename, settings);
  notifyActiveTabDownload(suggestedFilename || url);
});

async function notifyActiveTabDownload(filename) {
  try {
    const [activeTab] = await chrome.tabs.query({ active: true, currentWindow: true });
    if (activeTab && activeTab.id && activeTab.url) {
      // Don't send messages to restricted extension or browser system URLs
      if (!activeTab.url.startsWith('chrome://') && !activeTab.url.startsWith('edge://') && !activeTab.url.startsWith('about:') && !activeTab.url.startsWith('moz-extension://')) {
        chrome.tabs.sendMessage(activeTab.id, {
          action: 'SHOW_DOWNLOAD_TOAST',
          filename: filename
        }).catch(() => {});
      }
    }
  } catch (_) {}
}

/**
 * Generate cryptographically secure unique ID
 */
function generateTaskId() {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return 'task_' + crypto.randomUUID().replace(/-/g, '').slice(0, 12);
  }
  return 'task_' + Date.now() + '_' + Math.random().toString(36).substr(2, 9);
}

/**
 * Sanitize filename against path traversal and illegal characters
 */
function sanitizeFilename(name) {
  if (!name || typeof name !== 'string') return null;
  let clean = name.trim();
  // Remove directory traversal sequences
  clean = clean.replace(/(\.\.[\/\\]|\.\.)/g, '');
  // Remove illegal Windows & Unix path characters
  clean = clean.replace(/[<>:"/\\|?*\x00-\x1F]/g, '_');
  // Truncate length
  if (clean.length > 255) {
    clean = clean.substring(clean.length - 255);
  }
  return clean || 'download.bin';
}

export async function createDownloadTask(url, filename, options = {}) {
  // Protocol validation: only allow http and https
  if (!url || typeof url !== 'string') {
    throw new Error('Invalid URL provided');
  }
  const trimmedUrl = url.trim();
  if (!trimmedUrl.startsWith('http://') && !trimmedUrl.startsWith('https://')) {
    throw new Error('Only HTTP and HTTPS downloads are supported');
  }

  const safeFilename = sanitizeFilename(filename);
  const connections = Math.min(Math.max(parseInt(options.connections, 10) || 8, 1), 32);

  const taskId = generateTaskId();
  const task = new WasmDownloadTask(taskId, trimmedUrl, safeFilename, {
    ...options,
    connections,
    onStatusChange: () => {
      updateBadge();
      persistTasks();
      checkKeepAliveAlarm();
    }
  });

  tasks.set(taskId, task);
  updateBadge();
  persistTasks();
  checkKeepAliveAlarm();

  // Start download in background
  task.start().catch((err) => {
    console.error(`[AURORA] Task ${taskId} failed:`, err);
  }).finally(() => {
    updateBadge();
    persistTasks();
    checkKeepAliveAlarm();
  });

  return taskId;
}

let blinkInterval = null;
let blinkPhase = false;

export function updateBadge() {
  const activeCount = Array.from(tasks.values()).filter(t => 
    t.status === 'Downloading' || 
    t.status === 'Probing' || 
    t.status === 'Assembling' || 
    t.status === 'Verifying' ||
    t.status === 'Queued'
  ).length;

  if (activeCount > 0) {
    const badgeText = activeCount > 99 ? '99+' : String(activeCount);
    chrome.action.setBadgeText({ text: badgeText });
    try {
      if (chrome.action.setBadgeTextColor) {
        chrome.action.setBadgeTextColor({ color: '#FFFFFF' });
      }
    } catch (_) {}

    if (!blinkInterval) {
      try { chrome.action.setBadgeBackgroundColor({ color: '#007AFF' }); } catch (_) {}
      blinkInterval = setInterval(() => {
        const stillActive = Array.from(tasks.values()).filter(t => 
          t.status === 'Downloading' || 
          t.status === 'Probing' || 
          t.status === 'Assembling' || 
          t.status === 'Verifying' ||
          t.status === 'Queued'
        ).length;

        if (stillActive === 0) {
          clearInterval(blinkInterval);
          blinkInterval = null;
          chrome.action.setBadgeText({ text: '' });
          return;
        }

        blinkPhase = !blinkPhase;
        const color = blinkPhase ? '#007AFF' : '#0058BA';
        try { chrome.action.setBadgeBackgroundColor({ color }); } catch (_) {}
      }, 700);
    }
  } else {
    if (blinkInterval) {
      clearInterval(blinkInterval);
      blinkInterval = null;
    }
    chrome.action.setBadgeText({ text: '' });
  }
}

// Handle messaging from Popup, Content Script, and Options
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  // Only the offscreen document owns these replies. A second response races it.
  if (['CREATE_OBJECT_URL', 'SAVE_BLOB_DOWNLOAD', 'REVOKE_OBJECT_URL', 'OFFSCREEN_PING'].includes(message.action)) return false;
  (async () => {
    try {
      switch (message.action) {
        case 'GET_ALL_DOWNLOADS': {
          const list = Array.from(tasks.values()).map(t => typeof t.getSnapshot === 'function' ? t.getSnapshot() : t);
          sendResponse({ success: true, downloads: list });
          break;
        }

        case 'ADD_DOWNLOAD': {
          const { settings = DEFAULT_SETTINGS } = await chrome.storage.local.get('settings');
          const taskId = await createDownloadTask(message.url, message.filename, {
            ...settings,
            connections: message.connections || settings.connections,
            schedulerType: message.schedulerType || settings.schedulerType,
            checksum: message.checksum,
            hashAlgo: message.hashAlgo
          });
          sendResponse({ success: true, taskId });
          break;
        }

        case 'REGISTER_INTERNAL_URL': {
          if (message.url) {
            internalDownloadUrls.add(message.url);
          }
          sendResponse({ success: true });
          break;
        }

        case 'PAUSE_DOWNLOAD': {
          const task = tasks.get(message.taskId);
          if (task) {
            task.pause();
            updateBadge();
            persistTasks();
            checkKeepAliveAlarm();
            sendResponse({ success: true });
          } else {
            sendResponse({ success: false, error: 'Task not found' });
          }
          break;
        }

        case 'RESUME_DOWNLOAD': {
          const task = tasks.get(message.taskId);
          if (task) {
            task.resume().catch(err => {
              task.errorMessage = err.message;
              task.status = 'Failed';
              persistTasks();
            });
            updateBadge();
            persistTasks();
            checkKeepAliveAlarm();
            sendResponse({ success: true });
          } else {
            sendResponse({ success: false, error: 'Task not found' });
          }
          break;
        }

        case 'CANCEL_DOWNLOAD': {
          const task = tasks.get(message.taskId);
          if (task) {
            task.cancel();
            tasks.delete(message.taskId);
            updateBadge();
            persistTasks();
            checkKeepAliveAlarm();
            sendResponse({ success: true });
          } else {
            sendResponse({ success: false, error: 'Task not found' });
          }
          break;
        }

        case 'DELETE_TASKS': {
          const taskIds = Array.isArray(message.taskIds) ? message.taskIds : [message.taskId].filter(Boolean);
          const deleteFiles = Boolean(message.deleteFiles);

          for (const taskId of taskIds) {
            const task = tasks.get(taskId);
            if (task) {
              if (task.status === 'Downloading' || task.status === 'Probing' || task.status === 'Assembling') {
                task.cancel();
              }
              if (deleteFiles && task.downloadId) {
                try {
                  await chrome.downloads.removeFile(task.downloadId);
                  await chrome.downloads.erase({ id: task.downloadId });
                } catch (e) {
                  console.warn(`[AURORA] Could not remove file for task ${taskId}:`, e);
                }
              }
              tasks.delete(taskId);
            }
          }
          updateBadge();
          persistTasks();
          checkKeepAliveAlarm();
          sendResponse({ success: true, deletedCount: taskIds.length });
          break;
        }

        case 'CLEAR_COMPLETED': {
          for (const [id, task] of tasks.entries()) {
            if (task.status === 'Completed' || task.status === 'Cancelled' || task.status === 'Failed') {
              tasks.delete(id);
            }
          }
          updateBadge();
          persistTasks();
          checkKeepAliveAlarm();
          sendResponse({ success: true });
          break;
        }

        default:
          sendResponse({ success: false, error: 'Unknown action' });
      }
    } catch (err) {
      sendResponse({ success: false, error: err.message });
    }
  })();

  return true; // Keep message channel open for async response
});
