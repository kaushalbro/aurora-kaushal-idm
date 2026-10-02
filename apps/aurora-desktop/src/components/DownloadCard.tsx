import React from 'react';
import {
  Play,
  Pause,
  FolderOpen,
  FileCheck,
  Copy,
  Trash2,
  Search,
  FileText,
  Film,
  Music,
  Archive,
  Terminal,
  FileCode,
  CheckCircle2,
  AlertCircle,
  Clock,
  Timer,
  Check,
} from 'lucide-react';
import { DownloadItemDto } from '../types';
import {
  formatBytes,
  formatSpeed,
  formatEta,
  formatDuration,
  getFileCategory,
} from '../utils/formatters';

interface DownloadCardProps {
  item: DownloadItemDto;
  isSelected: boolean;
  onToggleSelect: (id: string) => void;
  onPause: (id: string) => void;
  onResume: (id: string) => void;
  onOpenFolder: () => void;
  onOpenFile: (path: string) => void;
  onCopyUrl: (url: string) => void;
  onRemove: (id: string) => void;
  onInspect: (item: DownloadItemDto) => void;
  onContextMenu: (e: React.MouseEvent, item: DownloadItemDto) => void;
}

export const DownloadCard: React.FC<DownloadCardProps> = ({
  item,
  isSelected,
  onToggleSelect,
  onPause,
  onResume,
  onOpenFolder,
  onOpenFile,
  onCopyUrl,
  onRemove,
  onInspect,
  onContextMenu,
}) => {
  const percent =
    item.total_bytes && item.total_bytes > 0
      ? Math.min(100, Math.round((item.downloaded_bytes / item.total_bytes) * 100))
      : item.status === 'completed'
      ? 100
      : 0;

  const category = getFileCategory(item.filename);

  const getFileIcon = () => {
    switch (category) {
      case 'video':
        return <Film className="w-5 h-5 text-[#007aff]" />;
      case 'audio':
        return <Music className="w-5 h-5 text-[#dc2626]" />;
      case 'documents':
        return <FileText className="w-5 h-5 text-[#d97706]" />;
      case 'compressed':
        return <Archive className="w-5 h-5 text-[#7c3aed]" />;
      case 'applications':
        return <Terminal className="w-5 h-5 text-[#16a34a]" />;
      default:
        return <FileCode className="w-5 h-5 text-[#374151]" />;
    }
  };

  const getStatusBadge = () => {
    if (item.status === 'downloading') {
      return (
        <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-[4px] text-[10.5px] font-bold bg-[#007aff]/15 text-[#007aff] uppercase tracking-wider">
          <span className="w-1.5 h-1.5 rounded-full bg-[#007aff] animate-pulse" />
          Downloading
        </span>
      );
    }
    if (item.status === 'completed') {
      return (
        <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-[4px] text-[10.5px] font-bold bg-[#16a34a]/15 text-[#16a34a] uppercase tracking-wider">
          <CheckCircle2 className="w-3.5 h-3.5" />
          Completed
        </span>
      );
    }
    if (item.status === 'paused') {
      return (
        <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-[4px] text-[10.5px] font-bold bg-[#d97706]/15 text-[#d97706] uppercase tracking-wider">
          Paused
        </span>
      );
    }
    return (
      <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-[4px] text-[10.5px] font-bold bg-[#dc2626]/15 text-[#dc2626] uppercase tracking-wider">
        <AlertCircle className="w-3.5 h-3.5" />
        {item.status.startsWith('failed') ? 'Failed' : item.status}
      </span>
    );
  };

  return (
    <div
      onContextMenu={(e) => onContextMenu(e, item)}
      className={`relative bg-white border rounded-[10px] p-3.5 transition-all duration-150 shadow-xs hover:bg-[#f9fafb] hover:shadow-sm ${
        isSelected
          ? 'border-[#007aff] bg-[#007aff]/[0.03] ring-1 ring-[#007aff]/40'
          : 'border-[rgba(0,0,0,0.1)]'
      }`}
    >
      {/* Top Header */}
      <div className="flex items-start justify-between gap-3 mb-2.5">
        <div className="flex items-center gap-2.5 min-w-0 flex-1">
          {/* Checkbox */}
          <input
            type="checkbox"
            checked={isSelected}
            onChange={() => onToggleSelect(item.id)}
            className="w-4 h-4 rounded-[3px] accent-[#007aff] cursor-pointer shrink-0"
          />

          {/* File Type Icon */}
          <div className="w-8 h-8 rounded-[6px] bg-[#f3f4f6] flex items-center justify-center shrink-0 border border-[rgba(0,0,0,0.06)]">
            {getFileIcon()}
          </div>

          {/* Name & URL */}
          <div className="min-w-0 flex-1">
            <div
              className="text-[13.5px] font-bold text-[#111827] truncate cursor-pointer hover:text-[#007aff]"
              title={item.filename}
              onClick={() => onInspect(item)}
            >
              {item.filename}
            </div>
            <div className="text-[11.5px] font-medium text-[#4b5563] truncate mt-0.5" title={item.url}>
              {item.url}
            </div>
          </div>
        </div>

        {/* Action Buttons */}
        <div className="flex items-center gap-1 shrink-0">
          {item.status === 'downloading' ? (
            <button
              onClick={() => onPause(item.id)}
              className="p-1.5 rounded-[5px] text-[#111827] bg-[#f3f4f6] hover:bg-[#e5e7eb] border border-[rgba(0,0,0,0.1)] cursor-pointer transition-all duration-120"
              title="Pause Download"
            >
              <Pause className="w-3.5 h-3.5 text-[#111827]" />
            </button>
          ) : item.status === 'paused' ? (
            <button
              onClick={() => onResume(item.id)}
              className="p-1.5 rounded-[5px] text-white bg-[#007aff] hover:bg-[#0066d6] cursor-pointer transition-all duration-120 shadow-xs"
              title="Resume Download"
            >
              <Play className="w-3.5 h-3.5 fill-white text-white" />
            </button>
          ) : null}

          {item.status === 'completed' && (
            <button
              onClick={() => onOpenFile(item.destination_path)}
              className="p-1.5 rounded-[5px] text-[#16a34a] bg-[#16a34a]/10 hover:bg-[#16a34a]/20 border border-[#16a34a]/30 cursor-pointer transition-all duration-120"
              title="Open File"
            >
              <FileCheck className="w-3.5 h-3.5" />
            </button>
          )}

          <button
            onClick={onOpenFolder}
            className="p-1.5 rounded-[5px] text-[#111827] bg-[#f3f4f6] hover:bg-[#e5e7eb] border border-[rgba(0,0,0,0.1)] cursor-pointer transition-all duration-120"
            title="Open Folder in File Explorer"
          >
            <FolderOpen className="w-3.5 h-3.5 text-[#111827]" />
          </button>

          <button
            onClick={() => onInspect(item)}
            className="p-1.5 rounded-[5px] text-[#111827] bg-[#f3f4f6] hover:bg-[#e5e7eb] border border-[rgba(0,0,0,0.1)] cursor-pointer transition-all duration-120"
            title="Diagnostics & HTTP Inspector"
          >
            <Search className="w-3.5 h-3.5 text-[#111827]" />
          </button>

          <button
            onClick={() => onCopyUrl(item.url)}
            className="p-1.5 rounded-[5px] text-[#111827] bg-[#f3f4f6] hover:bg-[#e5e7eb] border border-[rgba(0,0,0,0.1)] cursor-pointer transition-all duration-120"
            title="Copy URL"
          >
            <Copy className="w-3.5 h-3.5 text-[#111827]" />
          </button>

          <button
            onClick={() => onRemove(item.id)}
            className="p-1.5 rounded-[5px] text-[#dc2626] bg-[#f3f4f6] hover:bg-[#fee2e2] hover:border-[#dc2626]/30 border border-[rgba(0,0,0,0.1)] cursor-pointer transition-all duration-120"
            title="Delete / Remove"
          >
            <Trash2 className="w-3.5 h-3.5 text-[#dc2626]" />
          </button>
        </div>
      </div>

      {/* Main Flat Progress Bar */}
      <div className="w-full h-1.5 bg-[#e5e7eb] rounded-full overflow-hidden mb-2">
        <div
          className={`h-full transition-all duration-200 rounded-full ${
            item.status === 'completed'
              ? 'bg-[#16a34a]'
              : item.status === 'paused'
              ? 'bg-[#d97706]'
              : item.status.startsWith('failed')
              ? 'bg-[#dc2626]'
              : 'bg-[#007aff]'
          }`}
          style={{ width: `${percent}%` }}
        />
      </div>

      {/* Multi-Segment Map Visualizer (if active multi-stream download) */}
      {item.segments && item.segments.length > 0 && (
        <div className="flex h-2 gap-0.5 bg-[#e5e7eb] rounded-[3px] p-0.5 overflow-hidden mb-2">
          {item.segments.map((seg) => {
            const segState = seg.state;
            return (
              <div
                key={seg.index}
                className={`flex-1 h-full rounded-[1px] transition-colors duration-150 ${
                  segState === 'completed'
                    ? 'bg-[#16a34a]'
                    : segState === 'downloading'
                    ? 'bg-[#007aff]'
                    : 'bg-[#d1d5db]'
                }`}
                title={`Stream #${seg.index + 1}: ${segState} (${formatBytes(seg.downloaded_bytes)})`}
              />
            );
          })}
        </div>
      )}

      {/* Primary Metrics Row (Size, Speed, Streams, Status) */}
      <div className="flex items-center justify-between text-[11.5px] text-[#374151] mt-1.5 pt-1">
        <div className="flex items-center gap-2">
          <span className="font-bold text-[#111827]">
            {formatBytes(item.downloaded_bytes)}
            {item.total_bytes ? ` / ${formatBytes(item.total_bytes)}` : ''}
          </span>
          <span className="text-[11px] font-semibold text-[#4b5563]">({percent}%)</span>
          {item.active_connections > 1 && (
            <span className="text-[10px] bg-[#f3f4f6] text-[#111827] font-bold px-1.5 py-0.5 rounded-md border border-[rgba(0,0,0,0.1)]">
              {item.active_connections} streams
            </span>
          )}
        </div>

        <div className="flex items-center gap-3">
          {item.status === 'downloading' && (
            <span className="font-bold text-[#007aff] text-[12px]">
              {formatSpeed(item.current_speed)}
            </span>
          )}

          {getStatusBadge()}
        </div>
      </div>

      {/* Secondary Timing Row (Started time, End time / Time Took, Remaining / ETA) */}
      <div className="flex items-center justify-between text-[11px] font-medium text-[#4b5563] mt-1 pt-1 border-t border-[rgba(0,0,0,0.06)]">
        <div className="flex items-center gap-3">
          <span className="flex items-center gap-1">
            <Clock className="w-3 h-3 text-[#374151]" />
            <span>Started: <strong className="text-[#111827]">{item.start_time_str || '--:--:--'}</strong></span>
          </span>

          {item.status === 'completed' && item.end_time_str && (
            <span className="flex items-center gap-1">
              <Check className="w-3 h-3 text-[#16a34a]" />
              <span>Ended: <strong className="text-[#111827]">{item.end_time_str}</strong></span>
            </span>
          )}
        </div>

        <div className="flex items-center gap-3">
          {item.status === 'downloading' && (
            <span className="flex items-center gap-1">
              <Timer className="w-3 h-3 text-[#007aff]" />
              <span>Remaining: <strong className="text-[#007aff]">{formatEta(item.eta_secs)}</strong></span>
            </span>
          )}

          {item.status === 'completed' && item.elapsed_duration_secs !== null && (
            <span className="flex items-center gap-1">
              <Timer className="w-3 h-3 text-[#16a34a]" />
              <span>Time Took: <strong className="text-[#111827]">{formatDuration(item.elapsed_duration_secs)}</strong></span>
            </span>
          )}

          {item.status === 'paused' && (
            <span className="text-[#d97706] font-semibold">
              Paused (ETA: {formatEta(item.eta_secs)})
            </span>
          )}
        </div>
      </div>

      {/* Checksum Result if verified */}
      {item.checksum_result && (
        <div className="mt-2 text-[11px] font-semibold text-[#111827] bg-[#f3f4f6] px-2.5 py-1 rounded-[5px] border border-[rgba(0,0,0,0.1)]">
          {item.checksum_result}
        </div>
      )}
    </div>
  );
};
