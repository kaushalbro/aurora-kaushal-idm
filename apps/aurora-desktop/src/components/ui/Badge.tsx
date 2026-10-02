import React from 'react';
import { clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

export interface BadgeProps extends React.HTMLAttributes<HTMLSpanElement> {
  variant?: 'blue' | 'green' | 'orange' | 'red' | 'gray' | 'purple';
  hasPulse?: boolean;
  icon?: React.ReactNode;
}

export const Badge: React.FC<BadgeProps> = ({
  children,
  variant = 'gray',
  hasPulse = false,
  icon,
  className,
  ...props
}) => {
  const variantStyles = {
    blue: 'bg-[#007aff]/10 text-[#007aff] border-[#007aff]/20',
    green: 'bg-[#34c759]/15 text-[#28a745] border-[#34c759]/25',
    orange: 'bg-[#ff9500]/15 text-[#d97706] border-[#ff9500]/25',
    red: 'bg-[#ff3b30]/15 text-[#dc2626] border-[#ff3b30]/25',
    purple: 'bg-[#5856d6]/15 text-[#5856d6] border-[#5856d6]/25',
    gray: 'bg-[#f2f2f7] text-[#86868b] border-[rgba(0,0,0,0.06)]',
  };

  const pulseColors = {
    blue: 'bg-[#007aff]',
    green: 'bg-[#34c759]',
    orange: 'bg-[#ff9500]',
    red: 'bg-[#ff3b30]',
    purple: 'bg-[#5856d6]',
    gray: 'bg-[#86868b]',
  };

  return (
    <span
      className={twMerge(
        clsx(
          'inline-flex items-center gap-1 px-2 py-0.5 rounded-[4px] text-[10.5px] font-bold uppercase tracking-wider border select-none',
          variantStyles[variant],
          className
        )
      )}
      {...props}
    >
      {hasPulse && (
        <span className={`w-1.5 h-1.5 rounded-full animate-pulse ${pulseColors[variant]}`} />
      )}
      {icon && <span className="shrink-0">{icon}</span>}
      {children}
    </span>
  );
};
