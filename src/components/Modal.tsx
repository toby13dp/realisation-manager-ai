/// Modal dialog.

import { type ReactNode } from 'react';
import { X } from 'lucide-react';
import { cn } from '@/lib/utils';

interface ModalProps {
  open: boolean;
  onClose: () => void;
  title?: string;
  children: ReactNode;
  size?: 'sm' | 'md' | 'lg' | 'xl' | 'full';
  footer?: ReactNode;
}

const SIZE_MAP: Record<NonNullable<ModalProps['size']>, string> = {
  sm: 'max-w-md',
  md: 'max-w-2xl',
  lg: 'max-w-4xl',
  xl: 'max-w-6xl',
  full: 'max-w-[95vw]',
};

export function Modal({ open, onClose, title, children, size = 'md', footer }: ModalProps) {
  if (!open) return null;
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 animate-fade-in">
      <div
        className="absolute inset-0 bg-surface-950/50 backdrop-blur-sm"
        onClick={onClose}
      />
      <div
        className={cn(
          'relative bg-white rounded-xl shadow-2xl w-full max-h-[90vh] flex flex-col animate-slide-up',
          SIZE_MAP[size],
        )}
      >
        {title && (
          <div className="flex items-center justify-between px-6 py-4 border-b border-surface-200">
            <h2 className="text-lg font-semibold text-surface-900">{title}</h2>
            <button onClick={onClose} className="text-surface-400 hover:text-surface-700">
              <X className="w-5 h-5" />
            </button>
          </div>
        )}
        <div className="flex-1 overflow-auto p-6">{children}</div>
        {footer && (
          <div className="px-6 py-4 border-t border-surface-200 flex items-center justify-end gap-3">
            {footer}
          </div>
        )}
      </div>
    </div>
  );
}
