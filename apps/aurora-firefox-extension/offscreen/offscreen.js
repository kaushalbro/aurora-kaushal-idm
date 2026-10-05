/**
 * AURORA Kaushal IDM - Offscreen Document Handler
 * Handles DOM-dependent operations (URL.createObjectURL) in a background DOM context.
 */
import { getBlob, deleteBlob } from '../core/idb-store.js';

console.log('[AURORA Offscreen] Offscreen coordinator initialized.');

const activeObjectUrls = new Map();

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.action === 'CREATE_OBJECT_URL' || message.action === 'SAVE_BLOB_DOWNLOAD') {
    handleCreateObjectUrl(message)
      .then(res => sendResponse(res))
      .catch(err => {
        console.error('[AURORA Offscreen] Create Object URL error:', err);
        sendResponse({ success: false, error: err.message || String(err) });
      });
    return true; // Asynchronous reply
  }

  if (message.action === 'REVOKE_OBJECT_URL') {
    handleRevokeObjectUrl(message)
      .then(res => sendResponse(res))
      .catch(err => {
        console.warn('[AURORA Offscreen] Revoke error:', err);
        sendResponse({ success: false, error: err.message || String(err) });
      });
    return true;
  }

  if (message.action === 'OFFSCREEN_PING') {
    sendResponse({ success: true, pong: true });
    return false;
  }
});

async function handleCreateObjectUrl({ taskId, filename }) {
  console.log(`[AURORA Offscreen] Retrieving blob for task ${taskId}...`);
  const blob = await getBlob(taskId);
  if (!blob) {
    throw new Error(`Blob not found in IDB storage for task ${taskId}`);
  }

  const objectUrl = URL.createObjectURL(blob);
  const targetFilename = filename || 'download.bin';
  activeObjectUrls.set(taskId, objectUrl);
  console.log(`[AURORA Offscreen] Created Object URL for task ${taskId} (${targetFilename})`);

  try {
    await chrome.runtime.sendMessage({
      action: 'REGISTER_INTERNAL_URL',
      url: objectUrl,
      filename: targetFilename,
      taskId
    });
  } catch (_) {}

  return { success: true, objectUrl };
}

async function handleRevokeObjectUrl({ taskId, objectUrl }) {
  const url = objectUrl || activeObjectUrls.get(taskId);
  if (url) {
    try {
      URL.revokeObjectURL(url);
    } catch (_) {}
    activeObjectUrls.delete(taskId);
  }
  if (taskId) {
    try {
      await deleteBlob(taskId);
    } catch (_) {}
  }
  return { success: true };
}
