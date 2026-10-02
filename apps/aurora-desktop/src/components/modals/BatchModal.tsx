import React, { useState } from 'react';
import { Modal } from '../ui/Modal';
import { Select } from '../ui/Select';
import { Button } from '../ui/Button';
import { Layers } from 'lucide-react';

interface BatchModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (urls: string[], connections: number) => Promise<unknown>;
}

export const BatchModal: React.FC<BatchModalProps> = ({ isOpen, onClose, onSubmit }) => {
  const [urlsText, setUrlsText] = useState('');
  const [connections, setConnections] = useState<number>(16);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const parsedUrls = urlsText
    .split('\n')
    .map((u) => u.trim())
    .filter((u) => u.startsWith('http://') || u.startsWith('https://'));

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (parsedUrls.length === 0) return;

    setIsSubmitting(true);
    try {
      await onSubmit(parsedUrls, connections);
      setUrlsText('');
      onClose();
    } catch (err) {
      console.error(err);
    } finally {
      setIsSubmitting(false);
    }
  };

  const connectionOptions = [
    { value: 4, label: '4 Streams per download' },
    { value: 8, label: '8 Streams per download' },
    { value: 16, label: '16 Streams per download (Recommended)' },
    { value: 32, label: '32 Streams (Extreme Speed)' },
  ];

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title="Batch Download URLs"
      maxWidth="lg"
      footer={
        <>
          <Button variant="secondary" onClick={onClose} disabled={isSubmitting}>
            Cancel
          </Button>
          <Button
            variant="primary"
            onClick={handleSubmit}
            isLoading={isSubmitting}
            disabled={parsedUrls.length === 0}
          >
            Start {parsedUrls.length} Downloads
          </Button>
        </>
      }
    >
      <form onSubmit={handleSubmit} className="flex flex-col gap-3">
        <div className="flex flex-col gap-1 text-[12px]">
          <label className="text-[11px] font-semibold text-[#86868b] flex items-center justify-between">
            <span>Paste URLs (one per line)</span>
            <span className="text-[#007aff] font-bold">{parsedUrls.length} valid links detected</span>
          </label>
          <textarea
            rows={7}
            placeholder={`https://example.com/file1.zip\nhttps://example.com/file2.iso\nhttps://example.com/file3.tar.gz`}
            value={urlsText}
            onChange={(e) => setUrlsText(e.target.value)}
            className="w-full bg-white text-[#1d1d1f] text-[12px]  p-3 rounded-[6px] border border-[#d1d1d6] placeholder-[#aeaeb2] outline-none transition-all duration-150 focus:border-[#007aff] focus:ring-2 focus:ring-[#007aff]/20 resize-none"
            autoFocus
          />
        </div>

        <Select
          label="Parallel Streams per Download"
          options={connectionOptions}
          value={connections}
          onChange={(e) => setConnections(Number(e.target.value))}
        />
      </form>
    </Modal>
  );
};
