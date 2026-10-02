import React, { useState, useEffect } from 'react';
import { Modal } from '../ui/Modal';
import { Input } from '../ui/Input';
import { Select } from '../ui/Select';
import { Button } from '../ui/Button';
import { EngineConfigDto } from '../../types';
import { FolderOpen, Cpu, Rocket, Minimize2, ClipboardCopy } from 'lucide-react';

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  config: EngineConfigDto | null;
  autostartEnabled: boolean;
  onToggleAutostart: (enable: boolean) => Promise<void>;
  clipboardSniffing: boolean;
  onToggleClipboardSniffing: (enable: boolean) => void;
  onSave: (config: EngineConfigDto) => Promise<unknown>;
}

export const SettingsModal: React.FC<SettingsModalProps> = ({
  isOpen,
  onClose,
  config,
  autostartEnabled,
  onToggleAutostart,
  clipboardSniffing,
  onToggleClipboardSniffing,
  onSave,
}) => {
  const [downloadDir, setDownloadDir] = useState('');
  const [initialConnections, setInitialConnections] = useState(16);
  const [maxConnections, setMaxConnections] = useState(32);
  const [writeBufferBytes, setWriteBufferBytes] = useState(262144);
  const [maxRetries, setMaxRetries] = useState(5);
  const [autostart, setAutostart] = useState(autostartEnabled);
  const [clipSniff, setClipSniff] = useState(clipboardSniffing);
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    if (config) {
      setDownloadDir(config.download_dir);
      setInitialConnections(config.initial_connections);
      setMaxConnections(config.max_connections);
      setWriteBufferBytes(config.write_buffer_bytes);
      setMaxRetries(config.max_retries);
    }
    setAutostart(autostartEnabled);
    setClipSniff(clipboardSniffing);
  }, [config, autostartEnabled, clipboardSniffing, isOpen]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsSaving(true);
    try {
      if (autostart !== autostartEnabled) {
        await onToggleAutostart(autostart);
      }
      if (clipSniff !== clipboardSniffing) {
        onToggleClipboardSniffing(clipSniff);
      }

      await onSave({
        download_dir: downloadDir,
        initial_connections: initialConnections,
        max_connections: maxConnections,
        chunk_size_bytes: 131072,
        write_buffer_bytes: writeBufferBytes,
        request_timeout_secs: 30,
        max_retries: maxRetries,
        autostart,
        clipboard_sniffing: clipSniff,
      });
      onClose();
    } catch (err) {
      console.error(err);
    } finally {
      setIsSaving(false);
    }
  };

  const bufferOptions = [
    { value: 65536, label: '64 KB (Low Memory)' },
    { value: 262144, label: '256 KB (Recommended Aligned System I/O)' },
    { value: 1048576, label: '1 MB (High Throughput SSD)' },
    { value: 4194304, label: '4 MB (NVMe Ultra Speed)' },
  ];

  const retryOptions = [
    { value: 3, label: '3 Retries' },
    { value: 5, label: '5 Retries (Recommended)' },
    { value: 10, label: '10 Retries (Unstable Connections)' },
  ];

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title="Engine Preferences"
      maxWidth="md"
      footer={
        <>
          <Button variant="secondary" onClick={onClose} disabled={isSaving}>
            Cancel
          </Button>
          <Button variant="primary" onClick={handleSubmit} isLoading={isSaving}>
            Save Preferences
          </Button>
        </>
      }
    >
      <form onSubmit={handleSubmit} className="flex flex-col gap-3.5">
        <Input
          label="Default Download Directory"
          requiredAsterisk
          value={downloadDir}
          onChange={(e) => setDownloadDir(e.target.value)}
          leftIcon={<FolderOpen className="w-4 h-4" />}
          helperText="Files will be placed in this folder with 0.0s instant finalization."
        />

        <div className="grid grid-cols-2 gap-3">
          <Input
            label="Default Connections"
            type="number"
            min={1}
            max={32}
            value={initialConnections}
            onChange={(e) => setInitialConnections(Number(e.target.value))}
            leftIcon={<Cpu className="w-4 h-4" />}
          />

          <Input
            label="Max Multi-Stream Concurrency"
            type="number"
            min={1}
            max={64}
            value={maxConnections}
            onChange={(e) => setMaxConnections(Number(e.target.value))}
          />
        </div>

        <div className="grid grid-cols-2 gap-3">
          <Select
            label="Disk Write Buffer"
            options={bufferOptions}
            value={writeBufferBytes}
            onChange={(e) => setWriteBufferBytes(Number(e.target.value))}
          />

          <Select
            label="Max Network Retries"
            options={retryOptions}
            value={maxRetries}
            onChange={(e) => setMaxRetries(Number(e.target.value))}
          />
        </div>

        {/* System & Integration Preferences */}
        <div className="pt-2 border-t border-[rgba(0,0,0,0.08)] flex flex-col gap-2.5">
          <span className="text-[11px] font-bold uppercase tracking-wider text-[#374151]">
            System & Browser Integration
          </span>

          <label className="flex items-start gap-2.5 p-2.5 rounded-[8px] bg-[#f9fafb] border border-[rgba(0,0,0,0.08)] cursor-pointer hover:bg-[#f3f4f6] transition-colors">
            <input
              type="checkbox"
              checked={autostart}
              onChange={(e) => setAutostart(e.target.checked)}
              className="mt-0.5 w-4 h-4 rounded accent-[#007aff] cursor-pointer"
            />
            <div className="flex flex-col">
              <div className="flex items-center gap-1.5 text-[12.5px] font-bold text-[#111827]">
                <Rocket className="w-3.5 h-3.5 text-[#007aff]" />
                <span>Launch on System Startup / Reboot</span>
              </div>
              <span className="text-[11px] text-[#4b5563] mt-0.5">
                Automatically start AURORA IDM in system tray when your computer boots up.
              </span>
            </div>
          </label>

          <label className="flex items-start gap-2.5 p-2.5 rounded-[8px] bg-[#f9fafb] border border-[rgba(0,0,0,0.08)] cursor-pointer hover:bg-[#f3f4f6] transition-colors">
            <input
              type="checkbox"
              checked={clipSniff}
              onChange={(e) => setClipSniff(e.target.checked)}
              className="mt-0.5 w-4 h-4 rounded accent-[#007aff] cursor-pointer"
            />
            <div className="flex flex-col">
              <div className="flex items-center gap-1.5 text-[12.5px] font-bold text-[#111827]">
                <ClipboardCopy className="w-3.5 h-3.5 text-[#16a34a]" />
                <span>Universal Clipboard Sniffer (Any Browser)</span>
              </div>
              <span className="text-[11px] text-[#4b5563] mt-0.5">
                Automatically detect copied download links (.zip, .exe, .iso, .mp4, etc.) from any browser without needing an extension.
              </span>
            </div>
          </label>
        </div>
      </form>
    </Modal>
  );
};
