import React, { useState, useMemo } from 'react';
import {
  DownloadItemDto,
  CategoryFilter,
  SortColumn,
  SortDirection,
} from '../types';
import { DownloadCard } from './DownloadCard';
import { getFileCategory } from '../utils/formatters';
import {
  Search,
  ArrowUpDown,
  ArrowUp,
  ArrowDown,
  DownloadCloud,
} from 'lucide-react';

interface DownloadListProps {
  downloads: DownloadItemDto[];
  selectedCategory: CategoryFilter;
  selectedIds: Set<string>;
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

export const DownloadList: React.FC<DownloadListProps> = ({
  downloads,
  selectedCategory,
  selectedIds,
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
  const [searchQuery, setSearchQuery] = useState('');
  const [sortColumn, setSortColumn] = useState<SortColumn>('time');
  const [sortDirection, setSortDirection] = useState<SortDirection>('desc');

  const handleSort = (col: SortColumn) => {
    if (sortColumn === col) {
      setSortDirection((prev) => (prev === 'asc' ? 'desc' : 'asc'));
    } else {
      setSortColumn(col);
      setSortDirection('desc');
    }
  };

  const filteredAndSorted = useMemo(() => {
    let list = downloads.filter((d) => {
      // Filter by category
      if (selectedCategory === 'active' && d.status !== 'downloading') return false;
      if (selectedCategory === 'completed' && d.status !== 'completed') return false;
      if (selectedCategory === 'paused' && d.status !== 'paused') return false;
      if (
        selectedCategory === 'failed' &&
        !d.status.startsWith('failed') &&
        d.status !== 'cancelled'
      )
        return false;

      if (['video', 'audio', 'documents', 'compressed', 'applications'].includes(selectedCategory)) {
        if (getFileCategory(d.filename) !== selectedCategory) return false;
      }

      // Filter by search query
      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase();
        return (
          d.filename.toLowerCase().includes(q) ||
          d.url.toLowerCase().includes(q) ||
          (d.destination_path && d.destination_path.toLowerCase().includes(q))
        );
      }

      return true;
    });

    // Sort list
    list.sort((a, b) => {
      let cmp = 0;
      switch (sortColumn) {
        case 'name':
          cmp = a.filename.localeCompare(b.filename);
          break;
        case 'size':
          cmp = (a.total_bytes || a.downloaded_bytes) - (b.total_bytes || b.downloaded_bytes);
          break;
        case 'status':
          cmp = a.status.localeCompare(b.status);
          break;
        case 'speed':
          cmp = a.current_speed - b.current_speed;
          break;
        case 'time':
          cmp = a.start_time_str.localeCompare(b.start_time_str);
          break;
        case 'eta':
          cmp = (a.eta_secs || 999999) - (b.eta_secs || 999999);
          break;
      }
      return sortDirection === 'asc' ? cmp : -cmp;
    });

    return list;
  }, [downloads, selectedCategory, searchQuery, sortColumn, sortDirection]);

  const renderSortIndicator = (col: SortColumn) => {
    if (sortColumn !== col) {
      return <ArrowUpDown className="w-3.5 h-3.5 text-[#6b7280]" />;
    }
    return sortDirection === 'asc' ? (
      <ArrowUp className="w-3.5 h-3.5 text-[#007aff]" />
    ) : (
      <ArrowDown className="w-3.5 h-3.5 text-[#007aff]" />
    );
  };

  return (
    <main className="flex-1 flex flex-col min-w-0 bg-[#f3f4f6] overflow-hidden">
      {/* Search & Sort Controls Bar */}
      <div className="flex items-center justify-between gap-3 px-4 py-2.5 bg-white border-b border-[rgba(0,0,0,0.1)] select-none shrink-0">
        {/* Search input */}
        <div className="relative flex-1 max-w-sm">
          <Search className="w-4 h-4 absolute left-2.5 top-1/2 -translate-y-1/2 text-[#4b5563]" />
          <input
            type="text"
            placeholder="Search downloads by name, url, or path..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full bg-[#f3f4f6] text-[#111827] text-[12.5px] font-medium pl-8.5 pr-3 py-1.5 rounded-[6px] border border-transparent focus:border-[#007aff] focus:bg-white outline-none transition-all duration-150 placeholder-[#6b7280]"
          />
        </div>

        {/* Sort Column Buttons */}
        <div className="flex items-center gap-1.5 text-[11.5px] font-bold text-[#374151]">
          <span className="text-[10.5px] uppercase font-bold mr-1 text-[#374151]">Sort:</span>
          {(['name', 'size', 'status', 'speed', 'time', 'eta'] as SortColumn[]).map((col) => (
            <button
              key={col}
              onClick={() => handleSort(col)}
              className={`flex items-center gap-1 px-2.5 py-1 rounded-[5px] cursor-pointer transition-all duration-120 capitalize ${
                sortColumn === col
                  ? 'bg-[#007aff]/15 text-[#007aff] font-bold border border-[#007aff]/30 shadow-xs'
                  : 'hover:bg-[#f3f4f6] text-[#374151] border border-transparent'
              }`}
            >
              <span>{col}</span>
              {renderSortIndicator(col)}
            </button>
          ))}
        </div>
      </div>

      {/* Cards List Container */}
      <div className="flex-1 p-3.5 flex flex-col gap-2.5 overflow-y-auto">
        {filteredAndSorted.length === 0 ? (
          <div className="flex-1 flex flex-col items-center justify-center py-16 text-center text-[#4b5563] select-none">
            <div className="w-14 h-14 rounded-full bg-white border border-[rgba(0,0,0,0.1)] flex items-center justify-center text-[#007aff] mb-3 shadow-xs">
              <DownloadCloud className="w-7 h-7 stroke-[1.8]" />
            </div>
            <p className="text-[14px] font-bold text-[#111827]">No downloads found</p>
            <p className="text-[12px] text-[#4b5563] mt-1 max-w-sm font-medium">
              {searchQuery
                ? 'No downloads match your search filter.'
                : 'Click "+ Add URL" or paste links to download with 16–32 stream acceleration.'}
            </p>
          </div>
        ) : (
          filteredAndSorted.map((item) => (
            <DownloadCard
              key={item.id}
              item={item}
              isSelected={selectedIds.has(item.id)}
              onToggleSelect={onToggleSelect}
              onPause={onPause}
              onResume={onResume}
              onOpenFolder={onOpenFolder}
              onOpenFile={onOpenFile}
              onCopyUrl={onCopyUrl}
              onRemove={onRemove}
              onInspect={onInspect}
              onContextMenu={onContextMenu}
            />
          ))
        )}
      </div>
    </main>
  );
};
