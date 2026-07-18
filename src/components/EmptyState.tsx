/// Empty state placeholder.

import { type ReactNode } from 'react';
import { type LucideIcon } from 'lucide-react';

interface EmptyStateProps {
  icon?: LucideIcon;
  title: string;
  description?: string;
  action?: ReactNode;
}

export function EmptyState({ icon: Icon, title, description, action }: EmptyStateProps) {
  return (
    <div className="flex flex-col items-center justify-center py-16 px-6 text-center">
      {Icon && (
        <div className="rounded-full bg-surface-100 p-4 mb-4">
          <Icon className="w-8 h-8 text-surface-400" />
        </div>
      )}
      <h3 className="text-lg font-semibold text-surface-900">{title}</h3>
      {description && (
        <p className="mt-2 text-sm text-surface-500 max-w-md">{description}</p>
      )}
      {action && <div className="mt-6">{action}</div>}
    </div>
  );
}
