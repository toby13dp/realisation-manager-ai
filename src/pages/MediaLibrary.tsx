/// Media Library page — virtualized grid + filters + bulk actions.

import { useEffect, useMemo, useState } from 'react';
import { useSearchParams } from 'react-router-dom';
import { open } from '@tauri-apps/plugin-dialog';
import {
  Filter, Loader2, FolderOpen, Sparkles, Star, Lock, Unlock, Trash2, X,
} from 'lucide-react';
import { MediaGrid } from '@/components/MediaGrid';
import { Modal } from '@/components/Modal';
import { LoadingSpinner } from '@/components/LoadingSpinner';
import { EmptyState } from '@/components/EmptyState';
import { ClassificationBadge } from '@/components/ClassificationBadge';
import { ConfidenceBadge } from '@/components/ConfidenceBadge';
import { useMediaList, useToggleStar, useSetClassification, useSetPrivacy, useDeleteMedia } from '@/hooks/useMedia';
import { useAnalyzeBatch } from '@/hooks/useAIAnalysis';
import { useMediaStore } from '@/store/mediaStore';
import { mediaService } from '@/services/mediaService';
import { scannerService } from '@/services/settingsService';
import { toast } from '@/store/toastStore';
import type { Classification, Media } from '@/types';
import { formatDate, formatBytes } from '@/lib/format';
import { assetUrl } from '@/services/tauri';

export function MediaLibrary() {
  const [searchParams, setSearchParams] = useSearchParams();
  const [filter, setFilter] = useState<{
    classification?: Classification;
    mediaType?: 'image' | 'video' | 'raw' | 'unknown';
    isPrivate?: boolean;
    isStarred?: boolean;
    isDuplicate?: boolean;
  }>({});
  const [scanning, setScanning] = useState(false);
  const [detail, setDetail] = useState<Media | null>(null);
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const selected = useMediaStore((s) => s.selectedIds);

  const query = searchParams.get('query') || undefined;

  const { data: media, isLoading, refetch } = useMediaList({
    ...filter,
    limit: 5000,
  });

  const toggleStar = useToggleStar();
  const setClassification = useSetClassification();
  const setPrivacy = useSetPrivacy();
  const deleteMedia = useDeleteMedia();
  const analyzeBatch = useAnalyzeBatch();

  // Run search query if present
  useEffect(() => {
    if (!query) return;
    mediaService.search({ query, limit: 5000 }).then((results) => {
      // Replace the cache manually
      useMediaStore.getState().setLastResult(results);
    });
  }, [query]);

  const displayMedia = useMemo(() => {
    if (query && useMediaStore.getState().lastResult.length > 0) {
      return useMediaStore.getState().lastResult;
    }
    return media ?? [];
  }, [media, query]);

  async function handleImport() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Kies een map om te importeren',
    });
    if (typeof selected !== 'string') return;

    setScanning(true);
    try {
      const result = await scannerService.scanFolder(selected);
      toast.success(
        `Import voltooid: ${result.inserted} nieuw, ${result.duplicates} duplicaten, ${result.failed} fouten`,
      );
      refetch();
    } catch (e) {
      toast.error(`Import mislukt: ${e}`);
    } finally {
      setScanning(false);
    }
  }

  async function handleAnalyzeSelected() {
    if (selectedIds.length === 0) {
      toast.warning('Selecteer eerst media om te analyseren');
      return;
    }
    toast.info(`AI-analyse gestart voor ${selectedIds.length} items`);
    try {
      await analyzeBatch.mutateAsync(selectedIds);
      setSelectedIds([]);
    } catch (e) {
      toast.error(`Analyse mislukt: ${e}`);
    }
  }

  function toggleFilter(key: keyof typeof filter, value: any) {
    setFilter((f) => {
      const next = { ...f };
      if (next[key] === value) {
        delete next[key];
      } else {
        (next as any)[key] = value;
      }
      return next;
    });
  }

  return (
    <div className="space-y-6">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold text-surface-900">Mediabibliotheek</h1>
          <p className="text-surface-500 mt-1">
            {displayMedia.length} items
            {selectedIds.length > 0 && ` · ${selectedIds.length} geselecteerd`}
          </p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={handleImport}
            disabled={scanning}
            className="btn-primary"
          >
            {scanning ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <FolderOpen className="w-4 h-4" />
            )}
            Map importeren
          </button>
        </div>
      </div>

      {/* Filters */}
      <div className="card p-4 flex flex-wrap items-center gap-2">
        <Filter className="w-4 h-4 text-surface-400" />
        <span className="text-sm text-surface-600 mr-2">Filters:</span>

        {(['business', 'private', 'unclassified'] as Classification[]).map((c) => (
          <button
            key={c}
            onClick={() => toggleFilter('classification', c)}
            className={`btn-sm px-3 py-1 rounded-full text-xs font-medium ${
              filter.classification === c
                ? 'bg-brand-600 text-white'
                : 'bg-surface-100 text-surface-700 hover:bg-surface-200'
            }`}
          >
            {c === 'business' ? 'Zakelijk' : c === 'private' ? 'Prive' : 'Ongeclassificeerd'}
          </button>
        ))}

        <div className="w-px h-5 bg-surface-200 mx-2" />

        <button
          onClick={() => toggleFilter('isStarred', true)}
          className={`btn-sm px-3 py-1 rounded-full text-xs font-medium ${
            filter.isStarred ? 'bg-yellow-400 text-yellow-900' : 'bg-surface-100 hover:bg-surface-200'
          }`}
        >
          ★ Sterren
        </button>

        <button
          onClick={() => toggleFilter('isDuplicate', true)}
          className={`btn-sm px-3 py-1 rounded-full text-xs font-medium ${
            filter.isDuplicate ? 'bg-blue-600 text-white' : 'bg-surface-100 hover:bg-surface-200'
          }`}
        >
          Duplicaten
        </button>

        <button
          onClick={() => toggleFilter('isPrivate', true)}
          className={`btn-sm px-3 py-1 rounded-full text-xs font-medium ${
            filter.isPrivate ? 'bg-purple-600 text-white' : 'bg-surface-100 hover:bg-surface-200'
          }`}
        >
          Prive-media
        </button>

        {(Object.keys(filter).length > 0 || query) && (
          <button
            onClick={() => {
              setFilter({});
              setSearchParams({});
              useMediaStore.getState().setLastResult([]);
            }}
            className="btn-ghost text-xs ml-auto"
          >
            <X className="w-3 h-3" />
            Wis filters
          </button>
        )}
      </div>

      {/* Bulk actions */}
      {selectedIds.length > 0 && (
        <div className="card p-3 flex items-center gap-2 bg-brand-50 border-brand-200">
          <span className="text-sm font-medium text-brand-800 mr-2">
            {selectedIds.length} geselecteerd:
          </span>
          <button onClick={handleAnalyzeSelected} className="btn-secondary text-xs">
            <Sparkles className="w-3 h-3" />
            AI-analyse
          </button>
          <button
            onClick={() => {
              selectedIds.forEach((id) => setClassification.mutate({ id, classification: 'business' }));
              setSelectedIds([]);
            }}
            className="btn-secondary text-xs"
          >
            Mark als zakelijk
          </button>
          <button
            onClick={() => {
              selectedIds.forEach((id) => setClassification.mutate({ id, classification: 'private' }));
              setSelectedIds([]);
            }}
            className="btn-secondary text-xs"
          >
            Mark als prive
          </button>
          <button
            onClick={() => {
              if (!confirm(`${selectedIds.length} media verwijderen? Dit kan niet ongedaan worden gemaakt.`)) return;
              selectedIds.forEach((id) => deleteMedia.mutate(id));
              setSelectedIds([]);
            }}
            className="btn-danger text-xs"
          >
            <Trash2 className="w-3 h-3" />
            Verwijder
          </button>
          <button onClick={() => setSelectedIds([])} className="btn-ghost text-xs ml-auto">
            Deselecteer
          </button>
        </div>
      )}

      {/* Grid */}
      {isLoading ? (
        <LoadingSpinner size={36} />
      ) : displayMedia.length === 0 ? (
        <EmptyState
          icon={FolderOpen}
          title="Geen media"
          description="Importeer een map om te beginnen. De app ondersteunt JPG, PNG, HEIC, MOV, MP4 en RAW-formatten."
          action={
            <button onClick={handleImport} disabled={scanning} className="btn-primary">
              <FolderOpen className="w-4 h-4" />
              Map importeren
            </button>
          }
        />
      ) : (
        <MediaGrid
          media={displayMedia}
          selectedIds={selectedIds}
          onCardClick={(m) =>
            setSelectedIds((ids) =>
              ids.includes(m.id) ? ids.filter((x) => x !== m.id) : [...ids, m.id],
            )
          }
          onCardDoubleClick={(m) => setDetail(m)}
          onToggleStar={(m) => toggleStar.mutate({ id: m.id, starred: !m.isStarred })}
        />
      )}

      {/* Detail modal */}
      <Modal
        open={!!detail}
        onClose={() => setDetail(null)}
        title={detail?.fileName}
        size="xl"
      >
        {detail && <MediaDetail media={detail} />}
      </Modal>
    </div>
  );
}

function MediaDetail({ media }: { media: Media }) {
  const src = media.thumbnailPath
    ? assetUrl(media.thumbnailPath)
    : assetUrl(media.filePath);

  return (
    <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <div className="bg-surface-900 rounded-lg overflow-hidden flex items-center justify-center">
        <img src={src} alt={media.fileName} className="max-h-[70vh] object-contain" />
      </div>
      <div className="space-y-4">
        <div>
          <div className="flex items-center gap-2 mb-2">
            <ClassificationBadge classification={media.classification} />
            <ConfidenceBadge value={media.classificationConfidence} />
            {media.privacyLocked && (
              <span className="badge bg-yellow-100 text-yellow-800">
                <Lock className="w-3 h-3 mr-1" /> Vergrendeld
              </span>
            )}
          </div>
          <h3 className="font-medium text-surface-900 break-all">{media.fileName}</h3>
        </div>

        <dl className="grid grid-cols-2 gap-2 text-sm">
          <div>
            <dt className="text-surface-500">Type</dt>
            <dd className="font-medium uppercase">{media.fileExtension}</dd>
          </div>
          <div>
            <dt className="text-surface-500">Grootte</dt>
            <dd className="font-medium">{formatBytes(media.fileSize)}</dd>
          </div>
          <div>
            <dt className="text-surface-500">Datum genomen</dt>
            <dd className="font-medium">{formatDate(media.dateTaken)}</dd>
          </div>
          <div>
            <dt className="text-surface-500">Geimporteerd</dt>
            <dd className="font-medium">{formatDate(media.dateImported)}</dd>
          </div>
          <div className="col-span-2">
            <dt className="text-surface-500">Pad</dt>
            <dd className="font-mono text-xs break-all bg-surface-50 p-2 rounded">
              {media.filePath}
            </dd>
          </div>
          {media.sourceFolder && (
            <div className="col-span-2">
              <dt className="text-surface-500">Bronmap</dt>
              <dd className="font-mono text-xs break-all">{media.sourceFolder}</dd>
            </div>
          )}
          {media.qualityScore != null && (
            <div>
              <dt className="text-surface-500">Kwaliteitsscore</dt>
              <dd className="font-medium">{Math.round(media.qualityScore * 100)}%</dd>
            </div>
          )}
        </dl>
      </div>
    </div>
  );
}
