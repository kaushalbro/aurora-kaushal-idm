/**
 * AURORA Kaushal IDM - IndexedDB Large Blob Store
 * Provides blob storage shared between Service Worker and Offscreen documents.
 */

const DB_NAME = 'aurora_idb_store';
const DB_VERSION = 1;
const STORE_NAME = 'blobs';

function openDb() {
  return new Promise((resolve, reject) => {
    if (typeof indexedDB === 'undefined') {
      return reject(new Error('IndexedDB is not supported in this environment'));
    }

    const request = indexedDB.open(DB_NAME, DB_VERSION);

    request.onupgradeneeded = (event) => {
      const db = event.target.result;
      if (!db.objectStoreNames.contains(STORE_NAME)) {
        db.createObjectStore(STORE_NAME);
      }
    };

    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}

/**
 * Save a Blob or ArrayBuffer to IndexedDB
 * @param {string} taskId
 * @param {Blob|ArrayBuffer} blob
 */
export async function saveBlob(taskId, blob) {
  const db = await openDb();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, 'readwrite');
    const store = tx.objectStore(STORE_NAME);
    const req = store.put(blob, taskId);

    // Resolve only after commit so the offscreen reader can see the blob.
    req.onerror = () => reject(req.error);
    tx.oncomplete = () => { db.close(); resolve(true); };
    tx.onabort = tx.onerror = () => {
      db.close();
      reject(tx.error);
    };
  });
}

/**
 * Retrieve a stored Blob from IndexedDB
 * @param {string} taskId
 * @returns {Promise<Blob|null>}
 */
export async function getBlob(taskId) {
  const db = await openDb();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, 'readonly');
    const store = tx.objectStore(STORE_NAME);
    const req = store.get(taskId);

    req.onsuccess = () => resolve(req.result || null);
    req.onerror = () => reject(req.error);
    tx.oncomplete = () => db.close();
    tx.onabort = tx.onerror = () => {
      db.close();
      reject(tx.error);
    };
  });
}

/**
 * Delete a Blob from IndexedDB after successful save/download
 * @param {string} taskId
 */
export async function deleteBlob(taskId) {
  const db = await openDb();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, 'readwrite');
    const store = tx.objectStore(STORE_NAME);
    const req = store.delete(taskId);

    // Resolve only after commit so the offscreen reader can see the blob.
    req.onerror = () => reject(req.error);
    tx.oncomplete = () => { db.close(); resolve(true); };
    tx.onabort = tx.onerror = () => {
      db.close();
      reject(tx.error);
    };
  });
}
