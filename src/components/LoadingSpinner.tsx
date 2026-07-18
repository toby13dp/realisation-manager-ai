/// Loading spinner.

import { Loader2 } from 'lucide-react';
import { cn } from '@/lib/utils';

interface LoadingSpinnerProps {
  size?: number;
  className?: string;
}

export function LoadingSpinner({ size = 24, className }: LoadingSpinnerProps) {
  return <Loader2 className={cn('animate-spin text-brand-600', className)} style={{ width: size, height: size }} />;
}

export function FullPageSpinner({ message }: { message?: string }) {
  return (
    <div className="flex flex-col items-center justify-center py-20 gap-3">
      <LoadingSpinner size={36} />
      {message && <p className="text-surface-500 text-sm">{message}</p>}
    </div>
  );
}
