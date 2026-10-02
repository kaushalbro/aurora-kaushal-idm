import React from 'react';
import { clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';
import { ChevronDown } from 'lucide-react';

export interface SelectOption {
  value: string | number;
  label: string;
}

export interface SelectProps extends React.SelectHTMLAttributes<HTMLSelectElement> {
  label?: string;
  requiredAsterisk?: boolean;
  options: SelectOption[];
  helperText?: string;
}

export const Select: React.FC<SelectProps> = ({
  label,
  requiredAsterisk,
  options,
  helperText,
  className,
  id,
  ...props
}) => {
  const selectId = id || (label ? label.toLowerCase().replace(/\s+/g, '-') : undefined);

  return (
    <div className="flex flex-col gap-1 w-full text-[12px]">
      {label && (
        <label htmlFor={selectId} className="text-[11px] font-semibold text-[#86868b] flex items-center gap-1">
          <span>{label}</span>
          {requiredAsterisk && <span className="text-[#ff3b30] font-bold">*</span>}
        </label>
      )}

      <div className="relative flex items-center w-full">
        <select
          id={selectId}
          className={twMerge(
            clsx(
              'w-full bg-white text-[#1d1d1f] text-[12.5px] pl-3 pr-8 py-2 rounded-[6px] border border-[#d1d1d6] outline-none appearance-none cursor-pointer transition-all duration-150',
              'focus:border-[#007aff] focus:ring-2 focus:ring-[#007aff]/20',
              className
            )
          )}
          {...props}
        >
          {options.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>

        <ChevronDown className="w-3.5 h-3.5 text-[#86868b] absolute right-2.5 top-1/2 -translate-y-1/2 pointer-events-none" />
      </div>

      {helperText && <span className="text-[11px] text-[#86868b]">{helperText}</span>}
    </div>
  );
};
