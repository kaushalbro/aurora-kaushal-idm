/**
 * AURORA Kaushal IDM - Options Controller
 */

const checkAutoCapture = document.getElementById('check-auto-capture');
const inputExtensions = document.getElementById('input-extensions');
const selectDefaultConns = document.getElementById('select-default-conns');
const selectDefaultScheduler = document.getElementById('select-default-scheduler');
const btnSaveOptions = document.getElementById('btn-save-options');
const saveAlert = document.getElementById('save-alert');

document.addEventListener('DOMContentLoaded', async () => {
  const { settings = {} } = await chrome.storage.local.get('settings');

  checkAutoCapture.checked = settings.autoCapture !== false;
  inputExtensions.value = (settings.interceptExtensions || [
    'zip', 'rar', '7z', 'tar', 'gz', 'bz2', 'xz', 'tgz', 'zst', 'lz4', 'iso', 'img', 'bin',
    'exe', 'msi', 'dmg', 'pkg', 'deb', 'rpm', 'apk', 'aab', 'appimage', 'jar', 'whl', 'crx', 'wasm',
    'mp4', 'mkv', 'avi', 'mov', 'wmv', 'flv', 'webm', 'mp3', 'flac', 'wav', 'aac', 'm4a', 'ogg', 'opus',
    'pdf', 'doc', 'docx', 'xls', 'xlsx', 'ppt', 'pptx', 'epub', 'mobi', 'csv', 'sqlite', 'db', 'sql', 'vmdk', 'torrent'
  ]).join(', ');
  selectDefaultConns.value = String(settings.connections || 8);
  selectDefaultScheduler.value = settings.schedulerType || 'aurora-ect';

  btnSaveOptions.addEventListener('click', saveSettings);
});

async function saveSettings() {
  const extensions = inputExtensions.value
    .split(',')
    .map(e => e.trim().toLowerCase().replace(/^\./, ''))
    .filter(Boolean);

  const updatedSettings = {
    autoCapture: checkAutoCapture.checked,
    interceptExtensions: extensions,
    connections: parseInt(selectDefaultConns.value, 10),
    schedulerType: selectDefaultScheduler.value,
  };

  await chrome.storage.local.set({ settings: updatedSettings });

  saveAlert.style.display = 'block';
  setTimeout(() => {
    saveAlert.style.display = 'none';
  }, 2500);
}
