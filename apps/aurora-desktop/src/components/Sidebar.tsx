import React from 'react';
import {
  Inbox,
  ArrowDownCircle,
  CheckCircle2,
  PauseCircle,
  AlertCircle,
  Film,
  Music,
  FileText,
  Archive,
  Terminal,
  Zap,
  BarChart2,
} from 'lucide-react';
import { CategoryFilter, DownloadItemDto } from '../types';
import { getFileCategory } from '../utils/formatters';

interface SidebarProps {
  selectedCategory: CategoryFilter;
  onSelectCategory: (cat: CategoryFilter) => void;
  downloads: DownloadItemDto[];
  totalSpeedStr: string;
  activeTasks: number;
  showGraph: boolean;
  onToggleGraph: () => void;
}

export const Sidebar: React.FC<SidebarProps> = ({
  selectedCategory,
  onSelectCategory,
  downloads,
  totalSpeedStr,
  activeTasks,
  showGraph,
  onToggleGraph,
}) => {
  const counts = {
    all: downloads.length,
    active: downloads.filter((d) => d.status === 'downloading').length,
    completed: downloads.filter((d) => d.status === 'completed').length,
    paused: downloads.filter((d) => d.status === 'paused').length,
    failed: downloads.filter((d) => d.status.startsWith('failed') || d.status === 'cancelled').length,
    video: downloads.filter((d) => getFileCategory(d.filename) === 'video').length,
    audio: downloads.filter((d) => getFileCategory(d.filename) === 'audio').length,
    documents: downloads.filter((d) => getFileCategory(d.filename) === 'documents').length,
    compressed: downloads.filter((d) => getFileCategory(d.filename) === 'compressed').length,
    applications: downloads.filter((d) => getFileCategory(d.filename) === 'applications').length,
  };

  const statusCategories = [
    { id: 'all' as CategoryFilter, label: 'All Tasks', icon: Inbox, count: counts.all },
    { id: 'active' as CategoryFilter, label: 'Downloading', icon: ArrowDownCircle, count: counts.active, color: 'text-[#007aff]' },
    { id: 'completed' as CategoryFilter, label: 'Completed', icon: CheckCircle2, count: counts.completed, color: 'text-[#16a34a]' },
    { id: 'paused' as CategoryFilter, label: 'Paused', icon: PauseCircle, count: counts.paused, color: 'text-[#d97706]' },
    { id: 'failed' as CategoryFilter, label: 'Failed', icon: AlertCircle, count: counts.failed, color: 'text-[#dc2626]' },
  ];

  const fileTypeCategories = [
    { id: 'video' as CategoryFilter, label: 'Video', icon: Film, count: counts.video, color: 'text-[#007aff]' },
    { id: 'audio' as CategoryFilter, label: 'Audio', icon: Music, count: counts.audio, color: 'text-[#dc2626]' },
    { id: 'documents' as CategoryFilter, label: 'Documents', icon: FileText, count: counts.documents, color: 'text-[#d97706]' },
    { id: 'compressed' as CategoryFilter, label: 'Compressed', icon: Archive, count: counts.compressed, color: 'text-[#7c3aed]' },
    { id: 'applications' as CategoryFilter, label: 'Applications', icon: Terminal, count: counts.applications, color: 'text-[#16a34a]' },
  ];

  return (
    <aside className="w-54 bg-[#fafafa] border-r border-[rgba(0,0,0,0.1)] flex flex-col py-3 px-2.5 select-none shrink-0 overflow-y-auto">
      {/* Top Integrated Stats & Telemetry Graph Button (above Status) */}
      <div className="flex flex-col gap-2 mb-3.5 pb-3 border-b border-[rgba(0,0,0,0.1)]">
        {/* Telemetry Graph Toggle Button (Top) */}
        <button
          onClick={onToggleGraph}
          className={`w-full flex items-center justify-center gap-1.5 py-1.5 px-2 rounded-[6px] text-[11.5px] font-semibold transition-all duration-150 border cursor-pointer ${
            showGraph
              ? 'bg-[#007aff]/15 border-[#007aff] text-[#007aff] shadow-xs'
              : 'bg-white border-[rgba(0,0,0,0.15)] text-[#374151] hover:text-[#111827] hover:bg-[#f3f4f6] shadow-xs'
          }`}
          title="Toggle 60s Bandwidth Telemetry Graph"
        >
          <BarChart2 className="w-3.5 h-3.5" />
          <span>{showGraph ? 'Hide Graph' : 'Telemetry Graph'}</span>
        </button>

        {/* Speed & Active Tasks Card (Below Button) */}
        <div className="bg-white border border-[rgba(0,0,0,0.08)] rounded-[8px] p-2.5 shadow-xs space-y-2">
          {/* Total Speed */}
          <div className="flex items-center justify-between">
            <span className="text-[10px] font-bold text-[#4b5563] uppercase tracking-wider">Speed</span>
            <span className="text-[13px] font-extrabold text-[#007aff] tracking-tight">{totalSpeedStr}</span>
          </div>

          {/* Active Tasks */}
          <div className="flex items-center justify-between">
            <span className="text-[10px] font-bold text-[#4b5563] uppercase tracking-wider">Active</span>
            <div className="flex items-center gap-1.5">
              <span className="text-[12px] font-bold text-[#111827]">{activeTasks}</span>
              {activeTasks > 0 && <span className="w-2 h-2 rounded-full bg-[#16a34a] animate-pulse" />}
            </div>
          </div>

          {/* Engine Status */}
          <div className="flex items-center justify-between pt-1 border-t border-[rgba(0,0,0,0.06)]">
            <span className="text-[10px] font-bold text-[#4b5563] uppercase tracking-wider">Engine</span>
            <div className="flex items-center gap-1 text-[#007aff] font-bold text-[11px]">
              <Zap className="w-3 h-3 fill-[#007aff]" />
              <span>AURORA ECT</span>
            </div>
          </div>
        </div>
      </div>

      {/* Navigation Sections */}
      <div className="flex flex-col">
        <div className="px-2.5 pb-1.5 text-[11px] font-bold uppercase tracking-wider text-[#374151]">
          Status
        </div>
        <nav className="flex flex-col gap-0.5 mb-4">
          {statusCategories.map((cat) => {
            const Icon = cat.icon;
            const isSelected = selectedCategory === cat.id;
            return (
              <button
                key={cat.id}
                onClick={() => onSelectCategory(cat.id)}
                className={`flex items-center justify-between px-2.5 py-1.5 rounded-[6px] text-[12.5px] font-semibold transition-all duration-150 cursor-pointer ${
                  isSelected
                    ? 'bg-[#007aff] text-white shadow-xs'
                    : 'text-[#1f2937] hover:bg-[#f3f4f6]'
                }`}
              >
                <div className="flex items-center gap-2 min-w-0">
                  <Icon className={`w-4 h-4 shrink-0 ${isSelected ? 'text-white' : cat.color || 'text-[#374151]'}`} />
                  <span className="truncate">{cat.label}</span>
                </div>
                <span
                  className={`text-[11px] font-bold px-1.5 py-0.2 rounded-full ${
                    isSelected
                      ? 'bg-white/20 text-white'
                      : 'text-[#4b5563] bg-[#f3f4f6]'
                  }`}
                >
                  {cat.count}
                </span>
              </button>
            );
          })}
        </nav>

        <div className="px-2.5 pb-1.5 text-[11px] font-bold uppercase tracking-wider text-[#374151]">
          File Types
        </div>
        <nav className="flex flex-col gap-0.5">
          {fileTypeCategories.map((cat) => {
            const Icon = cat.icon;
            const isSelected = selectedCategory === cat.id;
            return (
              <button
                key={cat.id}
                onClick={() => onSelectCategory(cat.id)}
                className={`flex items-center justify-between px-2.5 py-1.5 rounded-[6px] text-[12.5px] font-semibold transition-all duration-150 cursor-pointer ${
                  isSelected
                    ? 'bg-[#007aff] text-white shadow-xs'
                    : 'text-[#1f2937] hover:bg-[#f3f4f6]'
                }`}
              >
                <div className="flex items-center gap-2 min-w-0">
                  <Icon className={`w-4 h-4 shrink-0 ${isSelected ? 'text-white' : cat.color || 'text-[#374151]'}`} />
                  <span className="truncate">{cat.label}</span>
                </div>
                <span
                  className={`text-[11px] font-bold px-1.5 py-0.2 rounded-full ${
                    isSelected
                      ? 'bg-white/20 text-white'
                      : 'text-[#4b5563] bg-[#f3f4f6]'
                  }`}
                >
                  {cat.count}
                </span>
              </button>
            );
          })}
        </nav>
      </div>
    </aside>
  );
};
