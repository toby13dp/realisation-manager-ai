/// Confidence badge.

import { cn } from '@/lib/utils';

interface ConfidenceBadgeProps {
  value: number;
  className?: string;
}

export function ConfidenceBadge({ value, className }: ConfidenceBadgeProps) {
  const pct = Math.round(value * 100);
  const color =
    value >= 0.85
      ? 'bg-green-100 text-green-800'
      : value >= 0.55
      ? 'bg-yellow-100 text-yellow-800'
      : 'bg-red-100 text-red-800';
  return (
    <span className={cn('badge', color, className)} title={`Confidence: ${pct}%`}>
      {pct}%
    </span>
  );
}
