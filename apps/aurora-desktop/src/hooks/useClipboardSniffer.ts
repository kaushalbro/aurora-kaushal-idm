import { useEffect, useRef } from 'react';

const DOWNLOAD_EXTENSION_REGEX = /\.(zip|rar|7z|tar|gz|bz2|xz|iso|dmg|exe|msi|deb|rpm|apk|appimage|pkg|mp4|mkv|mov|avi|webm|flv|mp3|m4a|flac|wav|ogg|pdf|epub|doc|docx|xls|xlsx|ppt|pptx|bin|img|csv|tsv|wasm)(\?.*)?$/i;

interface UseClipboardSnifferOptions {
  enabled: boolean;
  onDetectDownloadUrl: (url: string) => void;
}

export function useClipboardSniffer({
  enabled,
  onDetectDownloadUrl,
}: UseClipboardSnifferOptions) {
  const lastProcessedUrlRef = useRef<string>('');

  useEffect(() => {
    if (!enabled) return;

    const checkClipboard = async () => {
      try {
        if (!document.hasFocus()) return;
        if (!navigator.clipboard || !navigator.clipboard.readText) return;

        const text = await navigator.clipboard.readText();
        if (!text || typeof text !== 'string') return;

        const trimmed = text.trim();
        if (
          (trimmed.startsWith('http://') || trimmed.startsWith('https://')) &&
          trimmed !== lastProcessedUrlRef.current
        ) {
          // Check if it matches a downloadable file extension or direct download pattern
          if (DOWNLOAD_EXTENSION_REGEX.test(trimmed)) {
            lastProcessedUrlRef.current = trimmed;
            onDetectDownloadUrl(trimmed);
          }
        }
      } catch (_) {
        // Clipboard read permission might not be granted yet or window not focused
      }
    };

    // Check immediately when window gains focus
    const handleFocus = () => {
      checkClipboard();
    };

    window.addEventListener('focus', handleFocus);

    // Periodic check while app is active
    const interval = setInterval(checkClipboard, 2000);

    return () => {
      window.removeEventListener('focus', handleFocus);
      clearInterval(interval);
    };
  }, [enabled, onDetectDownloadUrl]);
}
