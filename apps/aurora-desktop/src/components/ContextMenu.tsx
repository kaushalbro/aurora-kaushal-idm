import React, { useEffect, useRef } from 'react';
import { DownloadItemDto } from '../types';
import {
  Play,
  Pause,
  RotateCcw,
  FileCheck,
  FolderOpen,
  Copy,
  Search,
  ShieldCheck,
  X,
  Trash2,
} from 'lucide-react';

interface ContextMenuProps {
  x: number;
  y: number;
  item: DownloadItemDto | null;
  onClose: () => void;
  onPause: (id: string) => void;
  onResume: (id: string) => void;
  onRestart: (id: string) => void;
  onOpenFile: (path: string) => void;
  onOpenFolder: () => void;
  onCopyUrl: (url: string) => void;
  onInspect: (item: DownloadItemDto) => void;
  onVerifyChecksum: (item: DownloadItemDto) => void;
  onRemove: (id: string, deleteFile: boolean) => void;
}

export const ContextMenu: React.FC<ContextMenuProps> = ({
  x,
  y,
  item,
  onClose,
  onPause,
  onResume,
  onRestart,
  onOpenFile,
  onOpenFolder,
  onCopyUrl,
  onInspect,
  onVerifyChecksum,
  onRemove,
}) => {
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        onClose();
      }
    };
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };

    window.addEventListener('mousedown', handleClickOutside);
    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('mousedown', handleClickOutside);
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [onClose]);

  if (!item) return null;

  // Position adjustment to stay within window bounds
  const adjustedX = Math.min(x, window.innerWidth - 220);
  const adjustedY = Math.min(y, window.innerHeight - 320);

  return (
    <div
      ref={menuRef}
      style={{ left: `${adjustedX}px`, top: `${adjustedY}px` }}
      className="fixed z-50 w-52 bg-white/95 backdrop-blur-md rounded-[8px] border border-[rgba(0,0,0,0.12)] shadow-xl p-1 select-none flex flex-col gap-0.5 text-[12px] text-[#1d1d1f] animate-in fade-in zoom-in-95 duration-100"
    >
      {item.status === 'downloading' ? (
        <button
          onClick={() => {
            onPause(item.id);
            onClose();
          }}
          className="flex items-center gap-2 px-2.5 py-1.5 rounded-[5px] hover:bg-[#007aff] hover:text-white cursor-pointer transition-colors duration-100"
        >
          <Pause className="w-3.5 h-3.5" />
          <span>Pause Download</span>
        </button>
      ) : item.status === 'paused' ? (
        <button
          onClick={() => {
            onResume(item.id);
            onClose();
          }}
          className="flex items-center gap-2 px-2.5 py-1.5 rounded-[5px] hover:bg-[#007aff] hover:text-white cursor-pointer transition-colors duration-100"
        >
          <Play className="w-3.5 h-3.5" />
          <span>Resume Download</span>
        </button>
      ) : null}

      <button
        onClick={() => {
          onRestart(item.id);
          onClose();
        }}
        className="flex items-center gap-2 px-2.5 py-1.5 rounded-[5px] hover:bg-[#007aff] hover:text-white cursor-pointer transition-colors duration-100"
      >
        <RotateCcw className="w-3.5 h-3.5" />
        <span>Restart from Beginning</span>
      </button>

      <div className="h-[1px] bg-[rgba(0,0,0,0.06)] my-0.5" />

      {item.status === 'completed' && (
        <button
          onClick={() => {
            onOpenFile(item.destination_path);
            onClose();
          }}
          className="flex items-center gap-2 px-2.5 py-1.5 rounded-[5px] hover:bg-[#007aff] hover:text-white cursor-pointer transition-colors duration-100"
        >
          <FileCheck className="w-3.5 h-3.5" />
          <span>Open File</span>
        </button>
      )}

      <button
        onClick={() => {
          onOpenFolder();
          onClose();
        }}
        className="flex items-center gap-2 px-2.5 py-1.5 rounded-[5px] hover:bg-[#007aff] hover:text-white cursor-pointer transition-colors duration-100"
      >
        <FolderOpen className="w-3.5 h-3.5" />
        <span>Open in File Explorer</span>
      </button>

      <button
        onClick={() => {
          onCopyUrl(item.url);
          onClose();
        }}
        className="flex items-center gap-2 px-2.5 py-1.5 rounded-[5px] hover:bg-[#007aff] hover:text-white cursor-pointer transition-colors duration-100"
      >
        <Copy className="w-3.5 h-3.5" />
        <span>Copy Download URL</span>
      </button>

      <button
        onClick={() => {
          onInspect(item);
          onClose();
        }}
        className="flex items-center gap-2 px-2.5 py-1.5 rounded-[5px] hover:bg-[#007aff] hover:text-white cursor-pointer transition-colors duration-100"
      >
        <Search className="w-3.5 h-3.5" />
        <span>Inspect Diagnostics</span>
      </button>

      <button
        onClick={() => {
          onVerifyChecksum(item);
          onClose();
        }}
        className="flex items-center gap-2 px-2.5 py-1.5 rounded-[5px] hover:bg-[#007aff] hover:text-white cursor-pointer transition-colors duration-100"
      >
        <ShieldCheck className="w-3.5 h-3.5" />
        <span>Verify Integrity Hash</span>
      </button>

      <div className="h-[1px] bg-[rgba(0,0,0,0.06)] my-0.5" />

      <button
        onClick={() => {
          onRemove(item.id, false);
          onClose();
        }}
        className="flex items-center gap-2 px-2.5 py-1.5 rounded-[5px] hover:bg-[#e5e5ea] text-[#1d1d1f] cursor-pointer transition-colors duration-100"
      >
        <X className="w-3.5 h-3.5" />
        <span>Remove from List</span>
      </button>

      <button
        onClick={() => {
          onRemove(item.id, true);
          onClose();
        }}
        className="flex items-center gap-2 px-2.5 py-1.5 rounded-[5px] hover:bg-[#ff3b30] hover:text-white text-[#ff3b30] cursor-pointer transition-colors duration-100"
      >
        <Trash2 className="w-3.5 h-3.5" />
        <span>Delete from Disk</span>
      </button>
    </div>
  );
};
