import React from 'react';
import { SegmentDto } from '../../types';
import { formatBytes } from '../../utils/formatters';

export interface SegmentMapProps {
  segments: SegmentDto[];
  className?: string;
}

export const SegmentMap: React.FC<SegmentMapProps> = ({ segments, className }) => {
  if (!segments || segments.length === 0) return null;

  return (
    <div
      className={`flex h-2 gap-0.5 bg-[#e5e5ea] rounded-[3px] p-0.5 overflow-hidden select-none ${
        className || ''
      }`}
    >
      {segments.map((seg) => {
        const segState = seg.state;
        return (
          <div
            key={seg.index}
            className={`flex-1 h-full rounded-[1px] transition-colors duration-150 ${
              segState === 'completed'
                ? 'bg-[#34c759]'
                : segState === 'downloading'
                ? 'bg-[#007aff]'
                : 'bg-[#d1d1d6]'
            }`}
            title={`Stream #${seg.index + 1}: ${segState} (${formatBytes(seg.downloaded_bytes)})`}
          />
        );
      })}
    </div>
  );
};
