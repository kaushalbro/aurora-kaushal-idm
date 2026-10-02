import React from 'react';
import { clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

export interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  requiredAsterisk?: boolean;
  error?: string;
  helperText?: string;
  leftIcon?: React.ReactNode;
  rightIcon?: React.ReactNode;
}

export const Input: React.FC<InputProps> = ({
  label,
  requiredAsterisk,
  error,
  helperText,
  leftIcon,
  rightIcon,
  className,
  id,
  ...props
}) => {
  const inputId = id || (label ? label.toLowerCase().replace(/\s+/g, '-') : undefined);

  return (
    <div className="flex flex-col gap-1 w-full text-[12px]">
      {label && (
        <label htmlFor={inputId} className="text-[11px] font-semibold text-[#86868b] flex items-center gap-1">
          <span>{label}</span>
          {requiredAsterisk && <span className="text-[#ff3b30] font-bold">*</span>}
        </label>
      )}

      <div className="relative flex items-center w-full">
        {leftIcon && (
          <div className="absolute left-2.5 top-1/2 -translate-y-1/2 text-[#86868b] pointer-events-none shrink-0">
            {leftIcon}
          </div>
        )}

        <input
          id={inputId}
          className={twMerge(
            clsx(
              'w-full bg-white text-[#1d1d1f] text-[12.5px] px-3 py-2 rounded-[6px] border border-[#d1d1d6] placeholder-[#aeaeb2] outline-none transition-all duration-150',
              'focus:border-[#007aff] focus:ring-2 focus:ring-[#007aff]/20',
              leftIcon && 'pl-8',
              rightIcon && 'pr-8',
              error && 'border-[#ff3b30] focus:border-[#ff3b30] focus:ring-[#ff3b30]/20',
              className
            )
          )}
          {...props}
        />

        {rightIcon && (
          <div className="absolute right-2.5 top-1/2 -translate-y-1/2 text-[#86868b] pointer-events-none shrink-0">
            {rightIcon}
          </div>
        )}
      </div>

      {error ? (
        <span className="text-[11px] text-[#ff3b30] font-medium">{error}</span>
      ) : helperText ? (
        <span className="text-[11px] text-[#86868b]">{helperText}</span>
      ) : null}
    </div>
  );
};
