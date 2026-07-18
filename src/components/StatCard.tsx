/// Stat card for dashboard.

import { type ReactNode } from 'react';
import { type LucideIcon } from 'lucide-react';
import { cn } from '@/lib/utils';

interface StatCardProps {
  label: string;
  value: string | number;
  icon?: LucideIcon;
  hint?: string;
  color?: 'brand' | 'green' | 'orange' | 'red' | 'gray';
}

const COLOR_MAP: Record<NonNullable<StatCardProps['color']>, string> = {
  brand: 'bg-brand-50 text-brand-700',
  green: 'bg-green-50 text-green-700',
  orange: 'bg-orange-50 text-orange-700',
  red: 'bg-red-50 text-red-700',
  gray: 'bg-surface-100 text-surface-700',
};

export function StatCard({ label, value, icon: Icon, hint, color = 'brand' }: StatCardProps) {
  return (
    <div className="card p-5 flex items-center gap-4">
      {Icon && (
        <div className={cn('rounded-lg p-3', COLOR_MAP[color])}>
          <Icon className="w-5 h-5" />
        </div>
      )}
      <div className="min-w-0">
        <p className="text-xs uppercase tracking-wide text-surface-500 font-medium">{label}</p>
        <p className="text-2xl font-bold text-surface-900 mt-0.5 truncate">{value}</p>
        {hint && <p className="text-xs text-surface-500 mt-1">{hint}</p>}
      </div>
    </div>
  );
}
