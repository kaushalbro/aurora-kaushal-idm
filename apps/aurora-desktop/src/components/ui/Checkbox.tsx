import React from 'react';
import { clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';
import { Check } from 'lucide-react';

export interface CheckboxProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label?: string;
  disabled?: boolean;
  className?: string;
}

export const Checkbox: React.FC<CheckboxProps> = ({
  checked,
  onChange,
  label,
  disabled,
  className,
}) => {
  return (
    <label
      className={twMerge(
        clsx(
          'inline-flex items-center gap-2 cursor-pointer select-none text-[12px] font-medium text-[#1d1d1f]',
          disabled && 'opacity-50 cursor-not-allowed pointer-events-none',
          className
        )
      )}
    >
      <div
        onClick={(e) => {
          if (!disabled) {
            e.preventDefault();
            onChange(!checked);
          }
        }}
        className={clsx(
          'w-4 h-4 rounded-[4px] border flex items-center justify-center transition-all duration-120 shrink-0',
          checked
            ? 'bg-[#007aff] border-[#007aff] text-white shadow-xs'
            : 'bg-white border-[rgba(0,0,0,0.25)] hover:border-[#007aff]'
        )}
      >
        {checked && <Check className="w-3 h-3 stroke-[3]" />}
      </div>
      {label && <span>{label}</span>}
    </label>
  );
};
