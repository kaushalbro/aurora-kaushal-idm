import React, { useState, useEffect, useRef } from 'react';
import { Modal } from '../ui/Modal';
import { Input } from '../ui/Input';
import { Select } from '../ui/Select';
import { Button } from '../ui/Button';
import { ServerCapabilitiesDto } from '../../types';
import { formatBytes } from '../../utils/formatters';
import { Globe, Zap, ShieldCheck, CheckCircle2, AlertTriangle, Loader2 } from 'lucide-react';

interface AddUrlModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (
    url: string,
    filename?: string,
    connections?: number,
    scheduler?: string,
    checksum?: string,
    algo?: string
  ) => Promise<unknown>;
  onProbeUrl: (url: string) => Promise<ServerCapabilitiesDto | null>;
}

export const AddUrlModal: React.FC<AddUrlModalProps> = ({
  isOpen,
  onClose,
  onSubmit,
  onProbeUrl,
}) => {
  const [url, setUrl] = useState('');
  const [filename, setFilename] = useState('');
  const [connections, setConnections] = useState<number>(16);
  const [scheduler, setScheduler] = useState('aurora-ect');
  const [checksum, setChecksum] = useState('');
  const [algo, setAlgo] = useState('sha256');

  const [isProbing, setIsProbing] = useState(false);
  const [probedCaps, setProbedCaps] = useState<ServerCapabilitiesDto | null>(null);
  const [probeError, setProbeError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const probeTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  // Auto-probe URL when user stops typing
  useEffect(() => {
    if (!isOpen) {
      setUrl('');
      setFilename('');
      setProbedCaps(null);
      setProbeError(null);
      return;
    }

    if (probeTimeoutRef.current) clearTimeout(probeTimeoutRef.current);

    if (url.trim().startsWith('http://') || url.trim().startsWith('https://')) {
      probeTimeoutRef.current = setTimeout(async () => {
        setIsProbing(true);
        setProbeError(null);
        try {
          const caps = await onProbeUrl(url.trim());
          setProbedCaps(caps);
          if (caps?.suggested_filename && !filename) {
            setFilename(caps.suggested_filename);
          }
        } catch (e: any) {
          setProbeError(e?.message || 'Failed to probe remote server');
        } finally {
          setIsProbing(false);
        }
      }, 500);
    } else {
      setProbedCaps(null);
      setProbeError(null);
    }

    return () => {
      if (probeTimeoutRef.current) clearTimeout(probeTimeoutRef.current);
    };
  }, [url, isOpen, onProbeUrl]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!url.trim()) return;

    setIsSubmitting(true);
    try {
      await onSubmit(
        url.trim(),
        filename.trim() || undefined,
        connections,
        scheduler,
        checksum.trim() || undefined,
        algo
      );
      onClose();
    } catch (err) {
      console.error(err);
    } finally {
      setIsSubmitting(false);
    }
  };

  const connectionOptions = [
    { value: 4, label: '4 Parallel Streams (Light)' },
    { value: 8, label: '8 Streams (Standard)' },
    { value: 16, label: '16 Streams (Recommended)' },
    { value: 32, label: '32 Streams (Extreme Turbo)' },
  ];

  const schedulerOptions = [
    { value: 'aurora-ect', label: 'AURORA ECT (Adaptive AI Scheduler)' },
    { value: 'largest-segment', label: 'Largest Segment (IDM-style)' },
    { value: 'fixed', label: 'Fixed Equal Partitioning' },
    { value: 'single', label: 'Single Stream (Baseline)' },
  ];

  const algoOptions = [
    { value: 'sha256', label: 'SHA-256' },
    { value: 'blake3', label: 'BLAKE3 (Ultra-Fast)' },
  ];

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title="Add New Download"
      maxWidth="md"
      footer={
        <>
          <Button variant="secondary" onClick={onClose} disabled={isSubmitting}>
            Cancel
          </Button>
          <Button
            variant="primary"
            onClick={handleSubmit}
            isLoading={isSubmitting}
            disabled={!url.trim()}
          >
            Start Download
          </Button>
        </>
      }
    >
      <form onSubmit={handleSubmit} className="flex flex-col gap-3">
        {/* URL Input */}
        <Input
          label="Download URL"
          requiredAsterisk
          placeholder="https://example.com/largefile.zip"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          autoFocus
          leftIcon={<Globe className="w-4 h-4" />}
          rightIcon={isProbing ? <Loader2 className="w-4 h-4 animate-spin text-[#007aff]" /> : undefined}
        />

        {/* Live Server Probe Indicator */}
        {probedCaps && (
          <div className="bg-[#f0f9ff] border border-[#bae6fd] rounded-[8px] p-2.5 text-[11.5px] text-[#0369a1] flex flex-col gap-1">
            <div className="flex items-center justify-between font-bold">
              <span className="flex items-center gap-1">
                <CheckCircle2 className="w-3.5 h-3.5 text-[#0284c7]" />
                Server Online & Verified
              </span>
              <span className="text-[#0369a1]  text-[11px]">
                {probedCaps.rtt_ms ? `${probedCaps.rtt_ms}ms RTT` : ''}
              </span>
            </div>
            <div className="flex items-center justify-between text-[11px] text-[#0c4a6e]">
              <span>Size: <strong>{formatBytes(probedCaps.content_length)}</strong></span>
              <span>Ranges: <strong>{probedCaps.accepts_ranges ? '✅ Supported (16-32 streams)' : '❌ Single stream only'}</strong></span>
              <span>Protocol: <strong>{probedCaps.http_version}</strong></span>
            </div>
          </div>
        )}

        {probeError && (
          <div className="bg-[#fff1f2] border border-[#fecdd3] rounded-[8px] p-2 text-[11px] text-[#be123c] flex items-center gap-1.5">
            <AlertTriangle className="w-3.5 h-3.5 shrink-0" />
            <span>{probeError}</span>
          </div>
        )}

        {/* Custom Filename */}
        <Input
          label="Custom Filename (Optional)"
          placeholder="Auto-detected from URL or server response"
          value={filename}
          onChange={(e) => setFilename(e.target.value)}
        />

        {/* Streams & Scheduler Row */}
        <div className="grid grid-cols-2 gap-3">
          <Select
            label="Parallel Connections"
            options={connectionOptions}
            value={connections}
            onChange={(e) => setConnections(Number(e.target.value))}
          />

          <Select
            label="Scheduling Algorithm"
            options={schedulerOptions}
            value={scheduler}
            onChange={(e) => setScheduler(e.target.value)}
          />
        </div>

        {/* Checksum & Hash Algorithm Row */}
        <div className="grid grid-cols-3 gap-3">
          <div className="col-span-2">
            <Input
              label="Expected Checksum (Optional)"
              placeholder="Hash hex string for integrity"
              value={checksum}
              onChange={(e) => setChecksum(e.target.value)}
              leftIcon={<ShieldCheck className="w-4 h-4" />}
            />
          </div>

          <Select
            label="Hash Algo"
            options={algoOptions}
            value={algo}
            onChange={(e) => setAlgo(e.target.value)}
          />
        </div>
      </form>
    </Modal>
  );
};
