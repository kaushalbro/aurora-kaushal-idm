import React, { useEffect } from 'react';
import { X } from 'lucide-react';
import { clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

export interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  children: React.ReactNode;
  footer?: React.ReactNode;
  maxWidth?: 'sm' | 'md' | 'lg' | 'xl';
  className?: string;
}

export const Modal: React.FC<ModalProps> = ({
  isOpen,
  onClose,
  title,
  children,
  footer,
  maxWidth = 'md',
  className,
}) => {
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isOpen) {
        onClose();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  const maxWidthStyles = {
    sm: 'max-w-sm',
    md: 'max-w-md',
    lg: 'max-w-lg',
    xl: 'max-w-2xl',
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/40 backdrop-blur-xs select-none animate-in fade-in duration-150">
      {/* Click outside backdrop */}
      <div className="absolute inset-0" onClick={onClose} />

      {/* Modal Dialog Box */}
      <div
        className={twMerge(
          clsx(
            'relative w-full bg-white rounded-[10px] border border-[rgba(0,0,0,0.12)] shadow-2xl overflow-hidden flex flex-col z-10 animate-in zoom-in-95 duration-150',
            maxWidthStyles[maxWidth],
            className
          )
        )}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-4 py-3 bg-[#f5f5f7] border-b border-[rgba(0,0,0,0.08)]">
          <h2 className="text-[13.5px] font-bold text-[#1d1d1f]">{title}</h2>
          <button
            onClick={onClose}
            className="p-1 rounded-[4px] text-[#86868b] hover:text-[#1d1d1f] hover:bg-[rgba(0,0,0,0.06)] cursor-pointer transition-colors duration-120"
            title="Close"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Body */}
        <div className="p-4 flex flex-col gap-3.5 max-h-[75vh] overflow-y-auto">{children}</div>

        {/* Footer */}
        {footer && (
          <div className="flex items-center justify-end gap-2 px-4 py-2.5 bg-[#f5f5f7] border-t border-[rgba(0,0,0,0.08)]">
            {footer}
          </div>
        )}
      </div>
    </div>
  );
};
