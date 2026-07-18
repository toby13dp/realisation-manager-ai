/// Toast container.

import { CheckCircle, AlertCircle, Info, AlertTriangle, X } from 'lucide-react';
import type { Toast as ToastType } from '@/store/toastStore';
import { cn } from '@/lib/utils';

interface ToastProps {
  toasts: ToastType[];
  onDismiss: (id: string) => void;
}

const ICON_MAP = {
  success: CheckCircle,
  error: AlertCircle,
  info: Info,
  warning: AlertTriangle,
} as const;

const COLOR_MAP = {
  success: 'bg-green-50 text-green-800 border-green-200',
  error: 'bg-red-50 text-red-800 border-red-200',
  info: 'bg-blue-50 text-blue-800 border-blue-200',
  warning: 'bg-yellow-50 text-yellow-800 border-yellow-200',
} as const;

export function Toast({ toasts, onDismiss }: ToastProps) {
  if (toasts.length === 0) return null;
  return (
    <div className="fixed bottom-6 right-6 z-50 flex flex-col gap-2 max-w-sm">
      {toasts.map((t) => {
        const Icon = ICON_MAP[t.type];
        return (
          <div
            key={t.id}
            className={cn(
              'flex items-start gap-3 px-4 py-3 rounded-lg border shadow-lg animate-slide-up',
              COLOR_MAP[t.type],
            )}
          >
            <Icon className="w-5 h-5 flex-shrink-0 mt-0.5" />
            <p className="text-sm flex-1">{t.message}</p>
            <button onClick={() => onDismiss(t.id)} className="opacity-60 hover:opacity-100">
              <X className="w-4 h-4" />
            </button>
          </div>
        );
      })}
    </div>
  );
}
