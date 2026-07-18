/// Classification badge.

import { cn } from '@/lib/utils';
import type { Classification } from '@/types';

const COLORS: Record<Classification, string> = {
  business: 'bg-brand-100 text-brand-800',
  private: 'bg-purple-100 text-purple-800',
  unclassified: 'bg-surface-100 text-surface-700',
  mixed: 'bg-orange-100 text-orange-800',
};

const LABELS: Record<Classification, string> = {
  business: 'Zakelijk',
  private: 'Prive',
  unclassified: 'Ongeclassificeerd',
  mixed: 'Gemengd',
};

interface ClassificationBadgeProps {
  classification: Classification;
  className?: string;
}

export function ClassificationBadge({ classification, className }: ClassificationBadgeProps) {
  return (
    <span className={cn('badge', COLORS[classification], className)}>
      {LABELS[classification]}
    </span>
  );
}
