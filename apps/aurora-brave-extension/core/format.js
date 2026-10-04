/** Bytes/s use binary units; network bits/s use decimal Mbps. */
export function formatSpeed(bytesPerSec) {
  if (!Number.isFinite(bytesPerSec) || bytesPerSec <= 0) return '0 B/s';
  const units = ['B/s', 'KiB/s', 'MiB/s', 'GiB/s', 'TiB/s'];
  const index = Math.min(units.length - 1, Math.max(0, Math.floor(Math.log2(bytesPerSec) / 10)));
  return `${(bytesPerSec / 1024 ** index).toFixed(index ? 2 : 0)} ${units[index]}`;
}

export function formatNetworkSpeed(bytesPerSec) {
  return `${(Math.max(0, Number.isFinite(bytesPerSec) ? bytesPerSec : 0) * 8 / 1_000_000).toFixed(2)} Mbps`;
}
