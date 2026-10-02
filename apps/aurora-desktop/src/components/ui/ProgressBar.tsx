import React from 'react';
import { clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

export interface ProgressBarProps {
  progress: number; // 0 to 100
  status?: 'downloading' | 'completed' | 'paused' | 'failed' | string;
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}

export const ProgressBar: React.FC<ProgressBarProps> = ({
  progress,
  status = 'downloading',
  size = 'md',
  className,
}) => {
  const clamped = Math.min(100, Math.max(0, progress));

  const sizeStyles = {
    sm: 'h-1',
    md: 'h-1.5',
    lg: 'h-2.5',
  };

  const getStatusBg = () => {
    if (status === 'completed') return 'bg-[#34c759]';
    if (status === 'paused') return 'bg-[#ff9500]';
    if (status.startsWith('failed') || status === 'cancelled') return 'bg-[#ff3b30]';
    return 'bg-[#007aff]';
  };

  return (
    <div
      className={twMerge(
        clsx(
          'w-full bg-[#e5e5ea] rounded-full overflow-hidden select-none',
          sizeStyles[size],
          className
        )
      )}
    >
      <div
        className={`h-full rounded-full transition-all duration-200 ease-out ${getStatusBg()}`}
        style={{ width: `${clamped}%` }}
      />
    </div>
  );
};
