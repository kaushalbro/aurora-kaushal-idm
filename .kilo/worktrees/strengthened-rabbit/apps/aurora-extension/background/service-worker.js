/**
 * AURORA Kaushal IDM - Service Worker (Manifest V3)
 * Universal Intelligent Download Interceptor
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
  interceptAllDownloads: true // Intercepts ALL browser downloads regardless of URL format
};

// Initialize context menus and settings on install
chrome.runtime.onInstalled.addListener(async () => {
  const current = await chrome.storage.local.get('settings');
  if (!current.settings) {
    await chrome.storage.local.set({ settings: DEFAULT_SETTINGS });
  }

  // Create Context Menus
  chrome.contextMenus.create({
    id: 'aurora-download-link',
    title: '⚡ Download with AURORA',
    contexts: ['link', 'image', 'video', 'audio', 'selection']
  });

  console.log('[AURORA] Service worker registered and context menus created.');
});

// Handle Context Menu clicks
chrome.contextMenus.onClicked.addListener(async (info, tab) => {
  const targetUrl = info.linkUrl || info.srcUrl || info.selectionText;
  if (!targetUrl) return;

  const { settings = DEFAULT_SETTINGS } = await chrome.storage.local.get('settings');
  await createDownloadTask(targetUrl.trim(), null, settings);
});

// Update badge on browser startup
chrome.runtime.onStartup.addListener(() => {
  updateBadge();
});

/**
 * Universal Download Interception:
 * Whenever Chrome begins ANY download (from buttons, dynamic links, redirects, or scripts),
 * AURORA intercepts it, cancels the slow single-stream browser download, and accelerates it with WASM.
 */
chrome.downloads.onCreated.addListener(async (downloadItem) => {
  const { settings = DEFAULT_SETTINGS } = await chrome.storage.local.get('settings');
  if (settings.autoCapture === false) return;

  const url = downloadItem.finalUrl || downloadItem.url;
  if (!url) return;

  // Ignore internal blob/data URLs or downloads triggered by AURORA itself
  if (url.startsWith('blob:') || url.startsWith('data:') || url.startsWith('chrome:') || internalDownloadUrls.has(url)) {
    return;
  }

  // Check MIME type: don't intercept accidental HTML "Save Page As" unless requested
  const mime = (downloadItem.mime || '').toLowerCase();
  if (mime === 'text/html' && !url.toLowerCase().includes('download') && !downloadItem.filename.includes('.')) {
    return;
  }

  console.log(`[AURORA] ⚡ Intercepted download: ${url} (MIME: ${mime}, Filename: ${downloadItem.filename})`);

  // 1. Cancel Chrome's slow native download immediately
  try {
    await chrome.downloads.cancel(downloadItem.id);
    await chrome.downloads.erase({ id: downloadItem.id });
  } catch (e) {
    console.warn('[AURORA] Could not cancel native download:', e);
  }

  // 2. Extract suggested filename if Chrome already knew it
  const suggestedFilename = downloadItem.filename ? downloadItem.filename.split(/[\/\\]/).pop() : null;

  // 3. Hand over to AURORA's high-speed WebAssembly Multi-Stream Downloader!
  await createDownloadTask(url, suggestedFilename, settings);
  notifyActiveTabDownload(suggestedFilename || url);
});

async function notifyActiveTabDownload(filename) {
  try {
    const [activeTab] = await chrome.tabs.query({ active: true, currentWindow: true });
    if (activeTab && activeTab.id) {
      chrome.tabs.sendMessage(activeTab.id, {
        action: 'SHOW_DOWNLOAD_TOAST',
        filename: filename
      }).catch(() => {});
    }
  } catch (_) {}
}

export async function createDownloadTask(url, filename, options = {}) {
  const taskId = 'task_' + Date.now() + '_' + Math.random().toString(36).substr(2, 6);
  const task = new WasmDownloadTask(taskId, url, filename, {
    ...options,
    onStatusChange: () => updateBadge()
  });
  tasks.set(taskId, task);
  updateBadge();

  // Start download in background
  task.start().catch((err) => {
    console.error(`[AURORA] Task ${taskId} failed:`, err);
  }).finally(() => {
    updateBadge();
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
    chrome.action.setBadgeText({ text: String(activeCount) });
    if (chrome.action.setBadgeTextColor) {
      chrome.action.setBadgeTextColor({ color: '#FFFFFF' });
    }

    if (!blinkInterval) {
      chrome.action.setBadgeBackgroundColor({ color: '#007AFF' });
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
        // Pulse badge color while download is running
        const color = blinkPhase ? '#007AFF' : '#0058BA';
        chrome.action.setBadgeBackgroundColor({ color });
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
  (async () => {
    try {
      switch (message.action) {
        case 'GET_ALL_DOWNLOADS': {
          const list = Array.from(tasks.values()).map(t => t.getSnapshot());
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
            sendResponse({ success: true });
          } else {
            sendResponse({ success: false, error: 'Task not found' });
          }
          break;
        }

        case 'RESUME_DOWNLOAD': {
          const task = tasks.get(message.taskId);
          if (task) {
            task.resume();
            updateBadge();
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
            sendResponse({ success: true });
          } else {
            sendResponse({ success: false, error: 'Task not found' });
          }
          break;
        }

        case 'CLEAR_COMPLETED': {
          for (const [id, task] of tasks.entries()) {
            if (task.status === 'Completed' || task.status === 'Cancelled' || task.status === 'Failed') {
              tasks.delete(id);
            }
          }
          updateBadge();
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
