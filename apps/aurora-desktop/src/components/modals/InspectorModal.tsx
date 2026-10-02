import React, { useState } from 'react';
import { Modal } from '../ui/Modal';
import { Input } from '../ui/Input';
import { Select } from '../ui/Select';
import { Button } from '../ui/Button';
import { Badge } from '../ui/Badge';
import { SegmentMap } from '../ui/SegmentMap';
import { DownloadItemDto } from '../../types';
import { formatBytes, formatSpeed, formatEta } from '../../utils/formatters';
import { ShieldCheck, Activity, Globe, Cpu, CheckCircle2, AlertTriangle, Layers } from 'lucide-react';

interface InspectorModalProps {
  isOpen: boolean;
  onClose: () => void;
  item: DownloadItemDto | null;
  onVerifyChecksum: (id: string, hash: string, algo: string) => Promise<string>;
}

export const InspectorModal: React.FC<InspectorModalProps> = ({
  isOpen,
  onClose,
  item,
  onVerifyChecksum,
}) => {
  const [expectedHash, setExpectedHash] = useState('');
  const [algo, setAlgo] = useState('sha256');
  const [verifyResult, setVerifyResult] = useState<string | null>(null);
  const [isVerifying, setIsVerifying] = useState(false);

  if (!item) return null;

  const handleVerify = async () => {
    if (!expectedHash.trim()) return;
    setIsVerifying(true);
    try {
      const res = await onVerifyChecksum(item.id, expectedHash.trim(), algo);
      setVerifyResult(res);
    } catch (e: any) {
      setVerifyResult(`❌ Verification error: ${e?.message || e}`);
    } finally {
      setIsVerifying(false);
    }
  };

  const algoOptions = [
    { value: 'sha256', label: 'SHA-256' },
    { value: 'blake3', label: 'BLAKE3 (Ultra-Fast)' },
  ];

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title={`Diagnostics: ${item.filename}`}
      maxWidth="lg"
      footer={
        <Button variant="secondary" onClick={onClose}>
          Close
        </Button>
      }
    >
      <div className="flex flex-col gap-3.5 text-[12px] select-none">
        {/* Top Summary Banner */}
        <div className="bg-[#fbfbfd] border border-[rgba(0,0,0,0.08)] rounded-[8px] p-3 flex flex-col gap-2">
          <div className="flex items-center justify-between">
            <span className="font-bold text-[13px] text-[#1d1d1f] truncate flex-1 mr-2">
              {item.filename}
            </span>
            <Badge
              variant={
                item.status === 'downloading'
                  ? 'blue'
                  : item.status === 'completed'
                  ? 'green'
                  : item.status === 'paused'
                  ? 'orange'
                  : 'red'
              }
              hasPulse={item.status === 'downloading'}
            >
              {item.status}
            </Badge>
          </div>

          <div className="text-[11px] text-[#86868b]  truncate">
            {item.destination_path}
          </div>

          <div className="grid grid-cols-4 gap-2 pt-1 border-t border-[rgba(0,0,0,0.06)] text-[11px]">
            <div>
              <span className="text-[#86868b] block">Downloaded:</span>
              <strong className="text-[#1d1d1f]">{formatBytes(item.downloaded_bytes)}</strong>
            </div>
            <div>
              <span className="text-[#86868b] block">Total Size:</span>
              <strong className="text-[#1d1d1f]">{formatBytes(item.total_bytes)}</strong>
            </div>
            <div>
              <span className="text-[#86868b] block">Speed:</span>
              <strong className="text-[#007aff]">{formatSpeed(item.current_speed)}</strong>
            </div>
            <div>
              <span className="text-[#86868b] block">ETA:</span>
              <strong className="text-[#1d1d1f]">{formatEta(item.eta_secs)}</strong>
            </div>
          </div>
        </div>

        {/* Multi-Segment Map */}
        {item.segments && item.segments.length > 0 && (
          <div className="flex flex-col gap-1">
            <span className="text-[11px] font-semibold text-[#86868b] uppercase tracking-wider flex items-center gap-1">
              <Layers className="w-3.5 h-3.5" />
              Parallel Stream Allocation ({item.segments.length} segments)
            </span>
            <SegmentMap segments={item.segments} />
          </div>
        )}

        {/* Server & Transport Capabilities */}
        <div className="bg-white border border-[rgba(0,0,0,0.08)] rounded-[8px] p-3 flex flex-col gap-2">
          <span className="text-[11px] font-bold text-[#1d1d1f] uppercase tracking-wider flex items-center gap-1 text-[#007aff]">
            <Globe className="w-3.5 h-3.5" />
            Server & Protocol Diagnostics
          </span>

          <div className="grid grid-cols-2 gap-2 text-[11.5px]">
            <div className="flex items-center justify-between p-1.5 rounded-[5px] bg-[#f2f2f7]">
              <span className="text-[#86868b]">HTTP Protocol:</span>
              <span className="font-semibold text-[#1d1d1f]">{item.http_version || 'HTTP/2'}</span>
            </div>

            <div className="flex items-center justify-between p-1.5 rounded-[5px] bg-[#f2f2f7]">
              <span className="text-[#86868b]">Accepts Ranges:</span>
              <span className="font-semibold text-[#1d1d1f]">
                {item.accepts_ranges ? '✅ Yes (Multi-stream enabled)' : '❌ No (Single stream)'}
              </span>
            </div>

            <div className="flex items-center justify-between p-1.5 rounded-[5px] bg-[#f2f2f7]">
              <span className="text-[#86868b]">Server Latency:</span>
              <span className="font-semibold text-[#1d1d1f]">
                {item.server_rtt_ms ? `${item.server_rtt_ms} ms RTT` : 'Direct Fast Connection'}
              </span>
            </div>

            <div className="flex items-center justify-between p-1.5 rounded-[5px] bg-[#f2f2f7]">
              <span className="text-[#86868b]">Active Streams:</span>
              <span className="font-semibold text-[#007aff]">{item.active_connections} streams</span>
            </div>

            {item.etag && (
              <div className="col-span-2 flex items-center justify-between p-1.5 rounded-[5px] bg-[#f2f2f7]  text-[10.5px]">
                <span className="text-[#86868b]">ETag:</span>
                <span className="font-semibold text-[#1d1d1f] truncate max-w-xs">{item.etag}</span>
              </div>
            )}
          </div>
        </div>

        {/* Cryptographic Hash Verification Tool */}
        <div className="bg-white border border-[rgba(0,0,0,0.08)] rounded-[8px] p-3 flex flex-col gap-2">
          <span className="text-[11px] font-bold text-[#1d1d1f] uppercase tracking-wider flex items-center gap-1 text-[#34c759]">
            <ShieldCheck className="w-3.5 h-3.5" />
            Cryptographic Integrity Verifier
          </span>

          <div className="grid grid-cols-3 gap-2">
            <div className="col-span-2">
              <Input
                placeholder="Paste expected SHA-256 or BLAKE3 hash hex"
                value={expectedHash}
                onChange={(e) => setExpectedHash(e.target.value)}
              />
            </div>

            <Select
              options={algoOptions}
              value={algo}
              onChange={(e) => setAlgo(e.target.value)}
            />
          </div>

          <div className="flex items-center justify-between mt-1">
            <Button
              variant="primary"
              size="sm"
              onClick={handleVerify}
              isLoading={isVerifying}
              disabled={!expectedHash.trim()}
              leftIcon={<ShieldCheck className="w-3.5 h-3.5" />}
            >
              Verify Checksum
            </Button>

            {verifyResult && (
              <span className="text-[11px] font-semibold text-[#1d1d1f]">{verifyResult}</span>
            )}
          </div>
        </div>
      </div>
    </Modal>
  );
};
