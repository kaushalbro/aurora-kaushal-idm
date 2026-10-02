import React, { useState, useEffect } from 'react';
import { Modal } from '../ui/Modal';
import { Input } from '../ui/Input';
import { Select } from '../ui/Select';
import { Button } from '../ui/Button';
import { EngineConfigDto } from '../../types';
import { FolderOpen, Cpu, HardDrive } from 'lucide-react';

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  config: EngineConfigDto | null;
  onSave: (config: EngineConfigDto) => Promise<unknown>;
}

export const SettingsModal: React.FC<SettingsModalProps> = ({
  isOpen,
  onClose,
  config,
  onSave,
}) => {
  const [downloadDir, setDownloadDir] = useState('');
  const [initialConnections, setInitialConnections] = useState(16);
  const [maxConnections, setMaxConnections] = useState(32);
  const [writeBufferBytes, setWriteBufferBytes] = useState(262144);
  const [maxRetries, setMaxRetries] = useState(5);
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    if (config) {
      setDownloadDir(config.download_dir);
      setInitialConnections(config.initial_connections);
      setMaxConnections(config.max_connections);
      setWriteBufferBytes(config.write_buffer_bytes);
      setMaxRetries(config.max_retries);
    }
  }, [config, isOpen]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsSaving(true);
    try {
      await onSave({
        download_dir: downloadDir,
        initial_connections: initialConnections,
        max_connections: maxConnections,
        chunk_size_bytes: 131072,
        write_buffer_bytes: writeBufferBytes,
        request_timeout_secs: 30,
        max_retries: maxRetries,
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
      </form>
    </Modal>
  );
};
