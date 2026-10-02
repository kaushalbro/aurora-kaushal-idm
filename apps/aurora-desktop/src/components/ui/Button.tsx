import React from 'react';
import { clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';
import { Loader2 } from 'lucide-react';

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'icon' | 'danger' | 'ghost';
  size?: 'sm' | 'md' | 'lg';
  isLoading?: boolean;
  leftIcon?: React.ReactNode;
  rightIcon?: React.ReactNode;
}

export const Button: React.FC<ButtonProps> = ({
  children,
  variant = 'secondary',
  size = 'md',
  isLoading = false,
  leftIcon,
  rightIcon,
  className,
  disabled,
  ...props
}) => {
  const baseStyles =
    'inline-flex items-center justify-center font-medium select-none cursor-pointer transition-all duration-150 rounded-[6px] outline-none disabled:opacity-50 disabled:cursor-not-allowed disabled:pointer-events-none active:scale-[0.98]';

  const sizeStyles = {
    sm: 'text-[11px] px-2 py-1 gap-1',
    md: 'text-[12px] px-3 py-1.5 gap-1.5',
    lg: 'text-[13px] px-4 py-2 gap-2 font-semibold',
  };

  const variantStyles = {
    primary:
      'bg-[#007aff] text-white hover:bg-[#0066d6] active:bg-[#0058ba] shadow-xs border border-transparent',
    secondary:
      'bg-white text-[#1d1d1f] border border-[rgba(0,0,0,0.12)] hover:bg-[#f2f2f7] hover:border-[rgba(0,0,0,0.2)] shadow-xs',
    icon: 'bg-white text-[#1d1d1f] border border-[rgba(0,0,0,0.12)] hover:bg-[#f2f2f7] hover:border-[rgba(0,0,0,0.2)] shadow-xs p-1.5',
    danger:
      'bg-white text-[#ff3b30] border border-[rgba(255,59,48,0.25)] hover:bg-[rgba(255,59,48,0.08)] hover:border-[#ff3b30] shadow-xs',
    ghost:
      'bg-transparent text-[#86868b] hover:text-[#1d1d1f] hover:bg-[#f2f2f7] border border-transparent',
  };

  return (
    <button
      className={twMerge(
        clsx(
          baseStyles,
          sizeStyles[size],
          variantStyles[variant],
          isLoading && 'cursor-wait',
          className
        )
      )}
      disabled={disabled || isLoading}
      {...props}
    >
      {isLoading ? (
        <Loader2 className="w-3.5 h-3.5 animate-spin" />
      ) : (
        leftIcon && <span className="shrink-0">{leftIcon}</span>
      )}
      {children && <span>{children}</span>}
      {!isLoading && rightIcon && <span className="shrink-0">{rightIcon}</span>}
    </button>
  );
};
