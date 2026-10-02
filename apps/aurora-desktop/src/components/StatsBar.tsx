import React from 'react';
import { Zap, BarChart2 } from 'lucide-react';

interface StatsBarProps {
  totalSpeedStr: string;
  activeTasks: number;
  showGraph: boolean;
  onToggleGraph: () => void;
}

export const StatsBar: React.FC<StatsBarProps> = ({
  totalSpeedStr,
  activeTasks,
  showGraph,
  onToggleGraph,
}) => {
  return (
    <section className="flex items-center justify-between px-6 py-2 bg-[#fafafa] border-b border-[rgba(0,0,0,0.1)] text-[12px] select-none shrink-0">
      <div className="flex items-center gap-6">
        <div className="flex flex-col">
          <span className="text-[10.5px] font-bold text-[#374151] uppercase tracking-wider">
            Total Speed
          </span>
          <span className="text-[13.5px] font-bold text-[#007aff] mt-0.5 tracking-tight">
            {totalSpeedStr}
          </span>
        </div>

        <div className="h-6 w-[1px] bg-[rgba(0,0,0,0.12)]" />

        <div className="flex flex-col">
          <span className="text-[10.5px] font-bold text-[#374151] uppercase tracking-wider">
            Active Tasks
          </span>
          <div className="flex items-center gap-1.5 mt-0.5">
            <span className="text-[13.5px] font-bold text-[#111827]">
              {activeTasks}
            </span>
            {activeTasks > 0 && (
              <span className="w-2 h-2 rounded-full bg-[#16a34a] animate-pulse" />
            )}
          </div>
        </div>

        <div className="h-6 w-[1px] bg-[rgba(0,0,0,0.12)]" />

        <div className="flex flex-col">
          <span className="text-[10.5px] font-bold text-[#374151] uppercase tracking-wider">
            Engine
          </span>
          <div className="flex items-center gap-1 mt-0.5 text-[#007aff] font-bold text-[12px]">
            <Zap className="w-3.5 h-3.5 fill-[#007aff]" />
            <span>AURORA ECT</span>
          </div>
        </div>
      </div>

      <div>
        <button
          onClick={onToggleGraph}
          className={`inline-flex items-center gap-1.5 px-3 py-1.5 rounded-[6px] text-[11.5px] font-semibold transition-all duration-150 border cursor-pointer ${
            showGraph
              ? 'bg-[#007aff]/15 border-[#007aff] text-[#007aff] shadow-xs'
              : 'bg-white border-[rgba(0,0,0,0.15)] text-[#374151] hover:text-[#111827] hover:bg-[#f3f4f6] shadow-xs'
          }`}
          title="Toggle 60s Bandwidth Telemetry Graph"
        >
          <BarChart2 className="w-3.5 h-3.5" />
          <span>{showGraph ? 'Hide Graph' : 'Telemetry Graph'}</span>
        </button>
      </div>
    </section>
  );
};
