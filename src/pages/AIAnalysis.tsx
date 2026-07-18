/// AI Analysis page — run batch analysis, see status.

import { useAiStatus, useAnalyzeBatch, useDetectProjects } from '@/hooks/useAIAnalysis';
import { useMediaList } from '@/hooks/useMedia';
import { LoadingSpinner } from '@/components/LoadingSpinner';
import { EmptyState } from '@/components/EmptyState';
import { MediaGrid } from '@/components/MediaGrid';
import {
  BrainCircuit, Cpu, ScanLine, Sparkles, Loader2,
  CheckCircle2, XCircle, AlertCircle, FolderSearch,
} from 'lucide-react';
import { useState } from 'react';
import { toast } from '@/store/toastStore';

export function AIAnalysis() {
  const { data: status, isLoading: statusLoading } = useAiStatus();
  const { data: unclassified } = useMediaList({
    classification: 'unclassified',
    isPrivate: false,
    limit: 1000,
  });
  const analyzeBatch = useAnalyzeBatch();
  const detect = useDetectProjects();
  const [selectedIds, setSelectedIds] = useState<string[]>([]);

  if (statusLoading || !status) {
    return <LoadingSpinner size={36} />;
  }

  async function handleAnalyzeAll() {
    if (!unclassified || unclassified.length === 0) {
      toast.warning('Geen ongeclassificeerde media om te analyseren');
      return;
    }
    const ids = unclassified.map((m) => m.id);
    toast.info(`AI-analyse gestart voor ${ids.length} items`);
    try {
      await analyzeBatch.mutateAsync(ids);
    } catch (e) {
      toast.error(`Analyse mislukt: ${e}`);
    }
  }

  async function handleAnalyzeSelected() {
    if (selectedIds.length === 0) {
      toast.warning('Selecteer eerst media');
      return;
    }
    await analyzeBatch.mutateAsync(selectedIds);
    setSelectedIds([]);
  }

  async function handleDetect() {
    try {
      const detected = await detect.mutateAsync(true);
      toast.success(`${detected.length} projecten gedetecteerd`);
    } catch (e) {
      toast.error(`Detectie mislukt: ${e}`);
    }
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-surface-900">AI-analyse</h1>
        <p className="text-surface-500 mt-1">
          Lokale AI-pipeline voor classificatie, objectdetectie en projectdetectie
        </p>
      </div>

      {/* AI Pipeline status */}
      <div className="card p-6">
        <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
          <Cpu className="w-5 h-5" />
          Pipeline status
        </h2>
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
          <PipelineStatusCard
            name="Objectdetectie"
            loaded={status.objectDetectionLoaded}
            model="YOLOv8 ONNX"
          />
          <PipelineStatusCard
            name="CLIP embeddings"
            loaded={status.clipLoaded}
            model="CLIP-ViT"
          />
          <PipelineStatusCard
            name="OCR"
            loaded={status.ocrAvailable}
            model="Tesseract"
          />
        </div>
        <div className="mt-4 text-sm text-surface-600">
          <p>
            <strong>Device:</strong> {status.device}
          </p>
          <p className="mt-1">
            <strong>Modelmap:</strong> <code className="text-xs">{status.modelDir}</code>
          </p>
        </div>
        {status.errors.length > 0 && (
          <div className="mt-4 rounded-lg bg-yellow-50 border border-yellow-200 p-3">
            <p className="text-sm font-medium text-yellow-800 flex items-center gap-2">
              <AlertCircle className="w-4 h-4" />
              Let op: sommige modellen zijn niet geladen
            </p>
            <ul className="mt-2 text-xs text-yellow-800 list-disc list-inside space-y-1">
              {status.errors.map((e, i) => (
                <li key={i}>{e}</li>
              ))}
            </ul>
            <p className="mt-2 text-xs text-yellow-700">
              Plaats ONNX-modelbestanden in de modelmap (zie Instellingen) en start de app opnieuw.
            </p>
          </div>
        )}
      </div>

      {/* Quick actions */}
      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <button
          onClick={handleAnalyzeAll}
          disabled={analyzeBatch.isPending || !unclassified || unclassified.length === 0}
          className="card p-5 text-left hover:shadow-md transition-shadow disabled:opacity-50"
        >
          <BrainCircuit className="w-8 h-8 text-brand-600 mb-3" />
          <h3 className="font-semibold text-surface-900">Analyseer alles</h3>
          <p className="text-sm text-surface-500 mt-1">
            {unclassified?.length ?? 0} ongeclassificeerde media analyseren
          </p>
        </button>

        <button
          onClick={handleAnalyzeSelected}
          disabled={analyzeBatch.isPending || selectedIds.length === 0}
          className="card p-5 text-left hover:shadow-md transition-shadow disabled:opacity-50"
        >
          <ScanLine className="w-8 h-8 text-brand-600 mb-3" />
          <h3 className="font-semibold text-surface-900">Analyseer selectie</h3>
          <p className="text-sm text-surface-500 mt-1">
            {selectedIds.length} geselecteerd
          </p>
        </button>

        <button
          onClick={handleDetect}
          disabled={detect.isPending}
          className="card p-5 text-left hover:shadow-md transition-shadow disabled:opacity-50"
        >
          <FolderSearch className="w-8 h-8 text-brand-600 mb-3" />
          <h3 className="font-semibold text-surface-900">Detecteer projecten</h3>
          <p className="text-sm text-surface-500 mt-1">
            Groepeer zakelijke media automatisch tot projecten
          </p>
        </button>
      </div>

      {/* Batch progress */}
      {analyzeBatch.isPending && (
        <div className="card p-4 bg-brand-50 border-brand-200 flex items-center gap-3">
          <Loader2 className="w-5 h-5 text-brand-600 animate-spin" />
          <p className="text-sm text-brand-800">
            Batch-analyse loopt... Bekijk voortgang in de jobs-tabel hieronder.
          </p>
        </div>
      )}

      {/* Uncategorized media */}
      <div>
        <h2 className="text-lg font-semibold mb-3">
          Ongeclassificeerde media ({unclassified?.length ?? 0})
        </h2>
        {unclassified && unclassified.length > 0 ? (
          <MediaGrid
            media={unclassified}
            selectedIds={selectedIds}
            onCardClick={(m) =>
              setSelectedIds((ids) =>
                ids.includes(m.id) ? ids.filter((x) => x !== m.id) : [...ids, m.id],
              )
            }
            emptyTitle="Alles is geclassificeerd"
            emptyDescription="Er is geen media meer die AI-analyse nodig heeft."
          />
        ) : (
          <EmptyState
            icon={Sparkles}
            title="Alles geclassificeerd"
            description="Er is momenteel geen ongeclassificeerde media. Importeer een nieuwe map om door te gaan."
          />
        )}
      </div>
    </div>
  );
}

function PipelineStatusCard({
  name,
  loaded,
  model,
}: {
  name: string;
  loaded: boolean;
  model: string;
}) {
  return (
    <div className="border border-surface-200 rounded-lg p-4">
      <div className="flex items-center justify-between mb-2">
        <span className="text-sm font-medium text-surface-700">{name}</span>
        {loaded ? (
          <CheckCircle2 className="w-5 h-5 text-green-600" />
        ) : (
          <XCircle className="w-5 h-5 text-red-500" />
        )}
      </div>
      <p className="text-xs text-surface-500">{model}</p>
      <p className={`text-xs mt-1 ${loaded ? 'text-green-700' : 'text-red-700'}`}>
        {loaded ? 'Beschikbaar' : 'Niet geladen'}
      </p>
    </div>
  );
}
