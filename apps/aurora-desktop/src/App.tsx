import React, { useState } from 'react';
import { useTauri } from './hooks/useTauri';
import { CategoryFilter, DownloadItemDto } from './types';
import { Header } from './components/Header';
import { SelectionBar } from './components/SelectionBar';
import { Sidebar } from './components/Sidebar';
import { DownloadList } from './components/DownloadList';
import { SpeedGraph } from './components/SpeedGraph';
import { ContextMenu } from './components/ContextMenu';
import { AddUrlModal } from './components/modals/AddUrlModal';
import { BatchModal } from './components/modals/BatchModal';
import { SettingsModal } from './components/modals/SettingsModal';
import { InspectorModal } from './components/modals/InspectorModal';

export const App: React.FC = () => {
  const {
    downloads,
    totalSpeedStr,
    activeTasks,
    speedHistory,
    config,
    toastMessage,
    showToast,
    addDownload,
    addBatchDownloads,
    pauseDownload,
    resumeDownload,
    cancelDownload,
    restartDownload,
    removeDownload,
    clearCompleted,
    probeUrl,
    verifyChecksum,
    openFolder,
    openFile,
    saveConfig,
  } = useTauri();

  // Navigation & Filtering
  const [selectedCategory, setSelectedCategory] = useState<CategoryFilter>('all');
  const [showSpeedGraph, setShowSpeedGraph] = useState<boolean>(false);

  // Multi-Selection State
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());

  // Modals state (coordinated so only 1 modal is open at a time)
  const [isAddModalOpen, setIsAddModalOpen] = useState(false);
  const [isBatchModalOpen, setIsBatchModalOpen] = useState(false);
  const [isSettingsModalOpen, setIsSettingsModalOpen] = useState(false);
  const [inspectingItem, setInspectingItem] = useState<DownloadItemDto | null>(null);

  // Context Menu State
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    item: DownloadItemDto;
  } | null>(null);

  // Modal helpers
  const openAddModal = () => {
    setIsAddModalOpen(true);
    setIsBatchModalOpen(false);
    setIsSettingsModalOpen(false);
    setInspectingItem(null);
  };

  const openBatchModal = () => {
    setIsBatchModalOpen(true);
    setIsAddModalOpen(false);
    setIsSettingsModalOpen(false);
    setInspectingItem(null);
  };

  const openSettingsModal = () => {
    setIsSettingsModalOpen(true);
    setIsAddModalOpen(false);
    setIsBatchModalOpen(false);
    setInspectingItem(null);
  };

  const openInspectorModal = (item: DownloadItemDto) => {
    setInspectingItem(item);
    setIsAddModalOpen(false);
    setIsBatchModalOpen(false);
    setIsSettingsModalOpen(false);
  };

  // Multi-selection handlers
  const handleToggleSelect = (id: string) => {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const handleSelectAll = () => {
    setSelectedIds(new Set(downloads.map((d) => d.id)));
  };

  const handleDeselectAll = () => {
    setSelectedIds(new Set());
  };

  const handleRemoveSelectedList = async () => {
    for (const id of Array.from(selectedIds)) {
      await removeDownload(id, false);
    }
    setSelectedIds(new Set());
  };

  const handleDeleteSelectedDisk = async () => {
    for (const id of Array.from(selectedIds)) {
      await removeDownload(id, true);
    }
    setSelectedIds(new Set());
  };

  // Right-click context menu handler
  const handleContextMenu = (e: React.MouseEvent, item: DownloadItemDto) => {
    e.preventDefault();
    setContextMenu({
      x: e.clientX,
      y: e.clientY,
      item,
    });
  };

  const handleCopyUrl = async (url: string) => {
    try {
      await navigator.clipboard.writeText(url);
      showToast('📋 Copied URL to clipboard');
    } catch {
      showToast('❌ Failed to copy to clipboard');
    }
  };

  return (
    <div className="flex flex-col w-screen h-screen bg-[#f5f5f7] text-[#1d1d1f] font-sans antialiased select-none overflow-hidden">
      {/* Top Header Toolbar in single clean horizontal row */}
      <Header
        onOpenAddModal={openAddModal}
        onOpenBatchModal={openBatchModal}
        onOpenFolder={openFolder}
        onClearCompleted={clearCompleted}
        onOpenSettings={openSettingsModal}
      />

      {/* 60s Bandwidth Spline Graph (Collapsible) */}
      {showSpeedGraph && <SpeedGraph speedHistory={speedHistory} />}

      {/* Bulk Selection Actions Bar */}
      <SelectionBar
        selectedCount={selectedIds.size}
        totalCount={downloads.length}
        onSelectAll={handleSelectAll}
        onDeselectAll={handleDeselectAll}
        onRemoveSelectedList={handleRemoveSelectedList}
        onDeleteSelectedDisk={handleDeleteSelectedDisk}
      />

      {/* Main Workspace: Sidebar (with integrated Live Stats & Graph Toggle) + Download List */}
      <div className="flex flex-1 min-h-0 overflow-hidden">
        <Sidebar
          selectedCategory={selectedCategory}
          onSelectCategory={setSelectedCategory}
          downloads={downloads}
          totalSpeedStr={totalSpeedStr}
          activeTasks={activeTasks}
          showGraph={showSpeedGraph}
          onToggleGraph={() => setShowSpeedGraph((prev) => !prev)}
        />

        <DownloadList
          downloads={downloads}
          selectedCategory={selectedCategory}
          selectedIds={selectedIds}
          onToggleSelect={handleToggleSelect}
          onPause={pauseDownload}
          onResume={resumeDownload}
          onOpenFolder={openFolder}
          onOpenFile={openFile}
          onCopyUrl={handleCopyUrl}
          onRemove={(id) => removeDownload(id, false)}
          onInspect={openInspectorModal}
          onContextMenu={handleContextMenu}
        />
      </div>

      {/* Floating Right-Click Context Menu */}
      {contextMenu && (
        <ContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          item={contextMenu.item}
          onClose={() => setContextMenu(null)}
          onPause={pauseDownload}
          onResume={resumeDownload}
          onRestart={restartDownload}
          onOpenFile={openFile}
          onOpenFolder={openFolder}
          onCopyUrl={handleCopyUrl}
          onInspect={openInspectorModal}
          onVerifyChecksum={openInspectorModal}
          onRemove={removeDownload}
        />
      )}

      {/* Centered Animated Dialog Modals */}
      <AddUrlModal
        isOpen={isAddModalOpen}
        onClose={() => setIsAddModalOpen(false)}
        onSubmit={addDownload}
        onProbeUrl={probeUrl}
      />

      <BatchModal
        isOpen={isBatchModalOpen}
        onClose={() => setIsBatchModalOpen(false)}
        onSubmit={addBatchDownloads}
      />

      <SettingsModal
        isOpen={isSettingsModalOpen}
        onClose={() => setIsSettingsModalOpen(false)}
        config={config}
        onSave={saveConfig}
      />

      <InspectorModal
        isOpen={!!inspectingItem}
        onClose={() => setInspectingItem(null)}
        item={inspectingItem}
        onVerifyChecksum={verifyChecksum}
      />

      {/* Toast Feedback Notification */}
      {toastMessage && (
        <div className="fixed bottom-5 right-5 z-50 bg-[#1d1d1f]/95 text-white text-[12px] font-semibold px-3.5 py-2 rounded-[8px] shadow-2xl backdrop-blur-md border border-white/10 animate-in fade-in slide-in-from-bottom-2 duration-150">
          {toastMessage}
        </div>
      )}
    </div>
  );
};
