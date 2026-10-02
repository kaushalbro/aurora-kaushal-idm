import React from 'react';
import { clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

export interface CardProps extends React.HTMLAttributes<HTMLDivElement> {
  isSelected?: boolean;
  hoverable?: boolean;
}

export const Card: React.FC<CardProps> = ({
  children,
  isSelected = false,
  hoverable = true,
  className,
  ...props
}) => {
  return (
    <div
      className={twMerge(
        clsx(
          'bg-white border rounded-[10px] p-3.5 transition-all duration-150 select-none shadow-xs',
          isSelected
            ? 'border-[#007aff] bg-[#007aff]/[0.025] ring-1 ring-[#007aff]/30 shadow-sm'
            : 'border-[rgba(0,0,0,0.08)]',
          hoverable && 'hover:bg-[#fafafc] hover:border-[rgba(0,0,0,0.14)] hover:shadow-sm',
          className
        )
      )}
      {...props}
    >
      {children}
    </div>
  );
};
