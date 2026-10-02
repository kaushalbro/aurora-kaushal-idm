import React from 'react';
import { Plus, FolderOpen, Trash2, Settings, Layers } from 'lucide-react';

interface HeaderProps {
  onOpenAddModal: () => void;
  onOpenBatchModal: () => void;
  onOpenFolder: () => void;
  onClearCompleted: () => void;
  onOpenSettings: () => void;
}

export const Header: React.FC<HeaderProps> = ({
  onOpenAddModal,
  onOpenBatchModal,
  onOpenFolder,
  onClearCompleted,
  onOpenSettings,
}) => {
  return (
    <header className="flex items-center justify-between px-5 py-2.5 bg-white border-b border-[rgba(0,0,0,0.1)] select-none shrink-0">
      {/* Left: Brand Logo & Title */}
      <div className="flex items-center gap-2.5">
        <img src="/icon.png" className="w-5 h-5 rounded-[4px] shadow-xs" alt="Aurora" />
        <h1 className="text-[13.5px] font-bold tracking-tight text-[#111827] whitespace-nowrap">
          Aurora Kaushal Download Manager <span className="text-[#007aff] font-bold text-[11px] ml-1 px-1.5 py-0.5 bg-[rgba(0,122,255,0.09)] rounded-md">Nepal</span>
        </h1>
      </div>

      {/* Right: Actions Toolbar in same single horizontal row */}
      <div className="flex items-center gap-2">
        <button
          onClick={onOpenAddModal}
          className="inline-flex items-center justify-center gap-1.5 px-3 py-1.5 rounded-[6px] text-[12px] font-semibold bg-[#007aff] text-white hover:bg-[#0066d6] active:bg-[#0058ba] shadow-xs cursor-pointer transition-all duration-150"
        >
          <Plus className="w-3.5 h-3.5 stroke-[2.4]" />
          <span>Add URL</span>
        </button>

        <button
          onClick={onOpenBatchModal}
          className="inline-flex items-center justify-center gap-1.5 px-3 py-1.5 rounded-[6px] text-[12px] font-semibold bg-white text-[#111827] border border-[rgba(0,0,0,0.15)] hover:bg-[#f3f4f6] shadow-xs cursor-pointer transition-all duration-150"
          title="Batch Add Multiple URLs"
        >
          <Layers className="w-3.5 h-3.5 text-[#111827]" />
          <span>Batch</span>
        </button>

        <button
          onClick={onOpenFolder}
          className="p-1.5 rounded-[6px] text-[#111827] bg-white border border-[rgba(0,0,0,0.15)] hover:bg-[#f3f4f6] shadow-xs cursor-pointer transition-all duration-150"
          title="Open Downloads Folder in File Explorer"
        >
          <FolderOpen className="w-4 h-4 text-[#111827]" />
        </button>

        <button
          onClick={onClearCompleted}
          className="p-1.5 rounded-[6px] text-[#111827] bg-white border border-[rgba(0,0,0,0.15)] hover:bg-[#f3f4f6] shadow-xs cursor-pointer transition-all duration-150"
          title="Clear Completed Downloads"
        >
          <Trash2 className="w-4 h-4 text-[#111827]" />
        </button>

        <button
          onClick={onOpenSettings}
          className="p-1.5 rounded-[6px] text-[#111827] bg-white border border-[rgba(0,0,0,0.15)] hover:bg-[#f3f4f6] shadow-xs cursor-pointer transition-all duration-150"
          title="Engine Settings"
        >
          <Settings className="w-4 h-4 text-[#111827]" />
        </button>
      </div>
    </header>
  );
};
