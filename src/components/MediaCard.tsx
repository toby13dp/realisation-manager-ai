/// Media card — shows a thumbnail with overlays for classification, quality,
/// brand, etc.

import { memo, useState } from 'react';
import { Star, Lock, Copy, Video, AlertTriangle } from 'lucide-react';
import type { Media } from '@/types';
import { ClassificationBadge } from './ClassificationBadge';
import { ConfidenceBadge } from './ConfidenceBadge';
import { assetUrl } from '@/services/tauri';
import { cn } from '@/lib/utils';
import { formatBytes } from '@/lib/format';

interface MediaCardProps {
  media: Media;
  selected?: boolean;
  onClick?: () => void;
  onDoubleClick?: () => void;
  onToggleStar?: () => void;
}

function MediaCardBase({ media, selected, onClick, onDoubleClick, onToggleStar }: MediaCardProps) {
  const [imgError, setImgError] = useState(false);

  const thumb = media.thumbnailPath
    ? assetUrl(media.thumbnailPath)
    : media.filePath
    ? assetUrl(media.filePath)
    : '';

  return (
    <div
      onClick={onClick}
      onDoubleClick={onDoubleClick}
      className={cn(
        'group relative aspect-square rounded-lg overflow-hidden cursor-pointer border-2 transition-all bg-surface-100',
        selected ? 'border-brand-500 ring-2 ring-brand-200' : 'border-transparent hover:border-brand-300',
      )}
    >
      {thumb && !imgError ? (
        <img
          src={thumb}
          alt={media.fileName}
          loading="lazy"
          onError={() => setImgError(true)}
          className="absolute inset-0 w-full h-full object-cover"
        />
      ) : (
        <div className="absolute inset-0 flex items-center justify-center text-surface-400">
          {media.mediaType === 'video' ? <Video className="w-8 h-8" /> : <AlertTriangle className="w-8 h-8" />}
        </div>
      )}

      {/* Top overlays */}
      <div className="absolute top-0 left-0 right-0 p-2 flex items-start justify-between gap-2 bg-gradient-to-b from-black/60 to-transparent opacity-0 group-hover:opacity-100 transition-opacity">
        <ClassificationBadge classification={media.classification} />
        <div className="flex items-center gap-1">
          {media.privacyLocked && <Lock className="w-3.5 h-3.5 text-yellow-300" />}
          {media.isDuplicate && <Copy className="w-3.5 h-3.5 text-blue-300" />}
          <button
            onClick={(e) => {
              e.stopPropagation();
              onToggleStar?.();
            }}
            className="text-white/80 hover:text-yellow-300"
          >
            <Star className={cn('w-4 h-4', media.isStarred && 'fill-yellow-400 text-yellow-400')} />
          </button>
        </div>
      </div>

      {/* Bottom overlays */}
      <div className="absolute bottom-0 left-0 right-0 p-2 flex items-end justify-between gap-2 bg-gradient-to-t from-black/70 to-transparent opacity-0 group-hover:opacity-100 transition-opacity">
        <p className="text-xs text-white truncate flex-1" title={media.fileName}>
          {media.fileName}
        </p>
        <div className="flex items-center gap-1 flex-shrink-0">
          {media.classification !== 'unclassified' && (
            <ConfidenceBadge value={media.classificationConfidence} />
          )}
        </div>
      </div>

      {/* Permanent indicator if starred */}
      {media.isStarred && (
        <div className="absolute top-2 right-2 group-hover:opacity-0 transition-opacity">
          <Star className="w-4 h-4 fill-yellow-400 text-yellow-400 drop-shadow" />
        </div>
      )}
    </div>
  );
}

export const MediaCard = memo(MediaCardBase);

export function MediaCardInfo({ media }: { media: Media }) {
  return (
    <div className="text-xs text-surface-500 space-y-0.5 mt-1">
      <p className="truncate font-medium text-surface-700">{media.fileName}</p>
      <div className="flex items-center gap-2">
        <span>{formatBytes(media.fileSize)}</span>
        <span>•</span>
        <span className="uppercase">{media.fileExtension}</span>
      </div>
    </div>
  );
}
