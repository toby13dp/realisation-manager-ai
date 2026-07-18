/// Virtualized media grid.
///
/// Renders thousands of items efficiently by only mounting what's visible
/// in the scroll viewport + a small overscan buffer.

import { useMemo, useRef, useState, useEffect, type UIEvent } from 'react';
import type { Media } from '@/types';
import { MediaCard } from './MediaCard';
import { EmptyState } from './EmptyState';
import { Images } from 'lucide-react';

interface MediaGridProps {
  media: Media[];
  selectedIds?: string[];
  onCardClick?: (media: Media) => void;
  onCardDoubleClick?: (media: Media) => void;
  onToggleStar?: (media: Media) => void;
  emptyTitle?: string;
  emptyDescription?: string;
  emptyAction?: React.ReactNode;
}

const COLUMN_MIN = 180;
const ROW_HEIGHT = 220; // includes gap
const OVERSCAN = 4;

export function MediaGrid({
  media,
  selectedIds = [],
  onCardClick,
  onCardDoubleClick,
  onToggleStar,
  emptyTitle = 'Geen media gevonden',
  emptyDescription = 'Importeer een map om media toe te voegen.',
  emptyAction,
}: MediaGridProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [cols, setCols] = useState(4);
  const [scrollTop, setScrollTop] = useState(0);
  const [viewportH, setViewportH] = useState(800);

  // Recompute column count on resize.
  useEffect(() => {
    if (!containerRef.current) return;
    const el = containerRef.current;
    const update = () => {
      const w = el.clientWidth;
      const c = Math.max(2, Math.floor(w / COLUMN_MIN));
      setCols(c);
      setViewportH(el.clientHeight);
    };
    update();
    const ro = new ResizeObserver(update);
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  const { rows, totalHeight, startRow, endRow } = useMemo(() => {
    const rows = Math.ceil(media.length / cols);
    const totalHeight = rows * ROW_HEIGHT;
    const startRow = Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - OVERSCAN);
    const visibleRows = Math.ceil(viewportH / ROW_HEIGHT) + OVERSCAN * 2;
    const endRow = Math.min(rows, startRow + visibleRows);
    return { rows, totalHeight, startRow, endRow };
  }, [media.length, cols, scrollTop, viewportH]);

  function onScroll(e: UIEvent<HTMLDivElement>) {
    setScrollTop(e.currentTarget.scrollTop);
  }

  if (media.length === 0) {
    return (
      <EmptyState
        icon={Images}
        title={emptyTitle}
        description={emptyDescription}
        action={emptyAction}
      />
    );
  }

  const visibleItems = media.slice(startRow * cols, endRow * cols);
  const offsetY = startRow * ROW_HEIGHT;

  return (
    <div
      ref={containerRef}
      onScroll={onScroll}
      className="overflow-auto h-full"
      style={{ maxHeight: 'calc(100vh - 200px)' }}
    >
      <div style={{ height: totalHeight, position: 'relative' }}>
        <div
          style={{
            transform: `translateY(${offsetY}px)`,
            display: 'grid',
            gridTemplateColumns: `repeat(${cols}, minmax(0, 1fr))`,
            gap: '12px',
            padding: '8px',
          }}
        >
          {visibleItems.map((m) => (
            <MediaCard
              key={m.id}
              media={m}
              selected={selectedIds.includes(m.id)}
              onClick={() => onCardClick?.(m)}
              onDoubleClick={() => onCardDoubleClick?.(m)}
              onToggleStar={() => onToggleStar?.(m)}
            />
          ))}
        </div>
      </div>
    </div>
  );
}
