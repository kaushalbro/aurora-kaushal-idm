import { useState, useEffect, useCallback } from 'react';
import { DownloadItemDto, EngineConfigDto, ServerCapabilitiesDto, TelemetryPayload } from '../types';

// Check if running inside native Tauri environment
const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

export function useTauri(callbacks?: {
  onExternalUrl?: (url: string) => void;
  onOpenAddModal?: () => void;
  onOpenSettingsModal?: () => void;
}) {
  const [downloads, setDownloads] = useState<DownloadItemDto[]>([]);
  const [totalSpeedStr, setTotalSpeedStr] = useState<string>('0.00 MB/s');
  const [totalSpeedBps, setTotalSpeedBps] = useState<number>(0);
  const [activeTasks, setActiveTasks] = useState<number>(0);
  const [speedHistory, setSpeedHistory] = useState<number[]>(new Array(60).fill(0));
  const [config, setConfig] = useState<EngineConfigDto | null>(null);
  const [autostartEnabled, setAutostartEnabled] = useState<boolean>(false);
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  const showToast = useCallback((msg: string) => {
    setToastMessage(msg);
    setTimeout(() => {
      setToastMessage((prev) => (prev === msg ? null : prev));
    }, 3200);
  }, []);

  // Safe wrapper for Tauri invoke
  const invokeTauri = useCallback(async <T>(cmd: string, args?: Record<string, unknown>): Promise<T | null> => {
    if (isTauri) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        return await invoke<T>(cmd, args);
      } catch (err) {
        console.error(`Tauri invoke error for ${cmd}:`, err);
        throw err;
      }
    } else {
      console.warn(`[Browser Preview] Mocking Tauri invoke for: ${cmd}`, args);
      return null;
    }
  }, []);

  // Command handlers for pause/resume all
  const pauseAll = useCallback(async () => {
    try {
      await invokeTauri('pause_all_downloads');
      showToast('⏸ Paused all active downloads');
    } catch (e: any) {
      showToast(`❌ Error pausing all: ${e?.toString() || e}`);
    }
  }, [invokeTauri, showToast]);

  const resumeAll = useCallback(async () => {
    try {
      await invokeTauri('resume_all_downloads');
      showToast('▶ Resumed all downloads');
    } catch (e: any) {
      showToast(`❌ Error resuming all: ${e?.toString() || e}`);
    }
  }, [invokeTauri, showToast]);

  // Subscribe to real-time events from Rust
  useEffect(() => {
    const unlistenFns: Array<() => void> = [];

    if (isTauri) {
      import('@tauri-apps/api/event').then(({ listen }) => {
        // Telemetry updates
        listen<TelemetryPayload>('telemetry-update', (event) => {
          const payload = event.payload;
          setDownloads(payload.downloads);
          setTotalSpeedStr(payload.total_speed_str);
          setTotalSpeedBps(payload.total_speed_bps);
          setActiveTasks(payload.active_tasks);
          setSpeedHistory(payload.speed_history);
        }).then((unlisten) => unlistenFns.push(unlisten));

        // Deep-link / CLI incoming URL (e.g. from browser or aurora:// protocol)
        listen<{ url: string }>('external-download-url', (event) => {
          if (event.payload?.url) {
            showToast(`🔗 Incoming download: ${event.payload.url}`);
            if (callbacks?.onExternalUrl) {
              callbacks.onExternalUrl(event.payload.url);
            }
          }
        }).then((unlisten) => unlistenFns.push(unlisten));

        // Tray actions
        listen('open-add-modal', () => {
          if (callbacks?.onOpenAddModal) callbacks.onOpenAddModal();
        }).then((unlisten) => unlistenFns.push(unlisten));

        listen('open-settings-modal', () => {
          if (callbacks?.onOpenSettingsModal) callbacks.onOpenSettingsModal();
        }).then((unlisten) => unlistenFns.push(unlisten));

        listen('tray-pause-all', () => {
          pauseAll();
        }).then((unlisten) => unlistenFns.push(unlisten));

        listen('tray-resume-all', () => {
          resumeAll();
        }).then((unlisten) => unlistenFns.push(unlisten));

        listen<boolean>('autostart-changed', (event) => {
          setAutostartEnabled(event.payload);
          showToast(event.payload ? '🚀 Auto-start on boot enabled' : 'Auto-start disabled');
        }).then((unlisten) => unlistenFns.push(unlisten));
      });
    }

    return () => {
      unlistenFns.forEach((fn) => fn());
    };
  }, [callbacks, pauseAll, resumeAll, showToast]);

  // Initial load
  const loadInitialData = useCallback(async () => {
    if (isTauri) {
      try {
        const initialDownloads = await invokeTauri<DownloadItemDto[]>('get_downloads');
        if (initialDownloads) setDownloads(initialDownloads);
        const cfg = await invokeTauri<EngineConfigDto>('get_config');
        if (cfg) setConfig(cfg);
        const auto = await invokeTauri<boolean>('get_autostart_status');
        if (typeof auto === 'boolean') setAutostartEnabled(auto);
      } catch (e) {
        console.error('Error fetching initial data:', e);
      }
    }
  }, [invokeTauri]);

  useEffect(() => {
    loadInitialData();
  }, [loadInitialData]);

  // Command handlers
  const addDownload = useCallback(
    async (
      url: string,
      filename?: string,
      connections?: number,
      scheduler?: string,
      checksum?: string,
      algo?: string
    ) => {
      try {
        const id = await invokeTauri<string>('add_download', {
          url,
          filename: filename || null,
          connections: connections || 16,
          scheduler: scheduler || 'aurora-ect',
          checksum: checksum || null,
          algo: algo || 'sha256',
        });
        showToast(`⚡ Download started with ${connections || 16} streams`);
        return id;
      } catch (e: any) {
        showToast(`❌ Failed to start: ${e?.toString() || e}`);
        throw e;
      }
    },
    [invokeTauri, showToast]
  );

  const addBatchDownloads = useCallback(
    async (urls: string[], connections?: number) => {
      try {
        const ids = await invokeTauri<string[]>('add_batch_downloads', {
          urls,
          connections: connections || 16,
        });
        showToast(`📦 Batch added ${ids?.length || urls.length} downloads`);
        return ids;
      } catch (e: any) {
        showToast(`❌ Batch error: ${e?.toString() || e}`);
        throw e;
      }
    },
    [invokeTauri, showToast]
  );

  const pauseDownload = useCallback(
    async (id: string) => {
      try {
        await invokeTauri('pause_download', { id });
        showToast('⏸ Download paused');
      } catch (e: any) {
        showToast(`❌ Error pausing: ${e?.toString() || e}`);
      }
    },
    [invokeTauri, showToast]
  );

  const resumeDownload = useCallback(
    async (id: string) => {
      try {
        await invokeTauri('resume_download', { id });
        showToast('▶ Resuming accelerated download...');
      } catch (e: any) {
        showToast(`❌ Error resuming: ${e?.toString() || e}`);
      }
    },
    [invokeTauri, showToast]
  );

  const cancelDownload = useCallback(
    async (id: string) => {
      try {
        await invokeTauri('cancel_download', { id });
        showToast('⏹ Download cancelled');
      } catch (e: any) {
        showToast(`❌ Error cancelling: ${e?.toString() || e}`);
      }
    },
    [invokeTauri, showToast]
  );

  const restartDownload = useCallback(
    async (id: string) => {
      try {
        await invokeTauri('restart_download', { id });
        showToast('🔄 Restarting download from beginning');
      } catch (e: any) {
        showToast(`❌ Error restarting: ${e?.toString() || e}`);
      }
    },
    [invokeTauri, showToast]
  );

  const removeDownload = useCallback(
    async (id: string, deleteFile: boolean = false) => {
      try {
        await invokeTauri('remove_download', { id, deleteFile });
        showToast(deleteFile ? '🗑 Removed & deleted from disk' : '🗑 Removed from list');
      } catch (e: any) {
        showToast(`❌ Error removing: ${e?.toString() || e}`);
      }
    },
    [invokeTauri, showToast]
  );

  const clearCompleted = useCallback(async () => {
    try {
      const removed = await invokeTauri<number>('clear_completed');
      showToast(`🧹 Cleared ${removed || 0} completed downloads`);
    } catch (e: any) {
      showToast(`❌ Error clearing: ${e?.toString() || e}`);
    }
  }, [invokeTauri, showToast]);

  const probeUrl = useCallback(
    async (url: string): Promise<ServerCapabilitiesDto | null> => {
      try {
        return await invokeTauri<ServerCapabilitiesDto>('probe_url', { url });
      } catch (e: any) {
        throw new Error(e?.toString() || 'Probe failed');
      }
    },
    [invokeTauri]
  );

  const verifyChecksum = useCallback(
    async (id: string, expectedHash: string, algo: string): Promise<string> => {
      try {
        const res = await invokeTauri<string>('verify_file_checksum', {
          id,
          expectedHash,
          algo,
        });
        return res || 'Integrity checked';
      } catch (e: any) {
        return `❌ Verification failed: ${e?.toString() || e}`;
      }
    },
    [invokeTauri]
  );

  const openFolder = useCallback(async () => {
    try {
      await invokeTauri('open_download_folder');
    } catch (e: any) {
      showToast(`❌ Error opening folder: ${e?.toString() || e}`);
    }
  }, [invokeTauri, showToast]);

  const openFile = useCallback(
    async (filepath: string) => {
      try {
        await invokeTauri('open_file', { filepath });
      } catch (e: any) {
        showToast(`❌ Error opening file: ${e?.toString() || e}`);
      }
    },
    [invokeTauri, showToast]
  );

  const setAutostart = useCallback(
    async (enable: boolean) => {
      try {
        const res = await invokeTauri<boolean>('set_autostart_status', { enabled: enable });
        setAutostartEnabled(Boolean(res));
        showToast(enable ? '🚀 Launch on system startup enabled' : 'Launch on system startup disabled');
      } catch (e: any) {
        showToast(`❌ Error setting autostart: ${e?.toString() || e}`);
      }
    },
    [invokeTauri, showToast]
  );

  const saveConfig = useCallback(
    async (newConfig: EngineConfigDto) => {
      try {
        await invokeTauri('update_config', { config: newConfig });
        setConfig(newConfig);
        showToast('⚙ Settings saved successfully');
      } catch (e: any) {
        showToast(`❌ Error saving settings: ${e?.toString() || e}`);
      }
    },
    [invokeTauri, showToast]
  );

  return {
    downloads,
    totalSpeedStr,
    totalSpeedBps,
    activeTasks,
    speedHistory,
    config,
    autostartEnabled,
    toastMessage,
    showToast,
    addDownload,
    addBatchDownloads,
    pauseDownload,
    resumeDownload,
    pauseAll,
    resumeAll,
    cancelDownload,
    restartDownload,
    removeDownload,
    clearCompleted,
    probeUrl,
    verifyChecksum,
    openFolder,
    openFile,
    setAutostart,
    saveConfig,
  };
}
