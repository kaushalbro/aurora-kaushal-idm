import React from 'react';
import { X, Trash2, CheckSquare, Square } from 'lucide-react';

interface SelectionBarProps {
  selectedCount: number;
  totalCount: number;
  onSelectAll: () => void;
  onDeselectAll: () => void;
  onRemoveSelectedList: () => void;
  onDeleteSelectedDisk: () => void;
}

export const SelectionBar: React.FC<SelectionBarProps> = ({
  selectedCount,
  totalCount,
  onSelectAll,
  onDeselectAll,
  onRemoveSelectedList,
  onDeleteSelectedDisk,
}) => {
  if (selectedCount === 0) return null;

  const allSelected = selectedCount === totalCount && totalCount > 0;

  return (
    <section className="flex items-center justify-between px-4 py-1.5 bg-[#f0f0f4] border-b border-[rgba(0,0,0,0.08)] text-[11.5px] select-none shrink-0 transition-all duration-150">
      <div className="flex items-center gap-2.5">
        <button
          onClick={allSelected ? onDeselectAll : onSelectAll}
          className="flex items-center gap-1.5 text-[#1d1d1f] font-medium cursor-pointer hover:text-[#007aff]"
        >
          {allSelected ? (
            <CheckSquare className="w-4 h-4 text-[#007aff]" />
          ) : (
            <Square className="w-4 h-4 text-[#86868b]" />
          )}
          <span className="font-semibold text-[#86868b]">{selectedCount} selected</span>
        </button>
      </div>

      <div className="flex items-center gap-2">
        <button
          onClick={onRemoveSelectedList}
          className="inline-flex items-center gap-1 px-2.5 py-1 rounded-[6px] text-[11px] font-medium bg-white text-[#1d1d1f] border border-[rgba(0,0,0,0.1)] hover:bg-[#e5e5ea] shadow-xs cursor-pointer transition-all duration-150"
          title="Remove selected items from list"
        >
          <X className="w-3.5 h-3.5" />
          <span>Remove from List</span>
        </button>

        <button
          onClick={onDeleteSelectedDisk}
          className="inline-flex items-center gap-1 px-2.5 py-1 rounded-[6px] text-[11px] font-medium bg-white text-[#ff3b30] border border-[rgba(255,59,48,0.2)] hover:bg-[rgba(255,59,48,0.1)] shadow-xs cursor-pointer transition-all duration-150"
          title="Delete selected files from disk"
        >
          <Trash2 className="w-3.5 h-3.5 text-[#ff3b30]" />
          <span>Delete from Disk</span>
        </button>
      </div>
    </section>
  );
};
