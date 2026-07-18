/// Settings page — general settings + AI model paths + folder rules.

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useEffect, useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import {
  Save, FolderOpen, Cpu, Database, FolderPlus, Trash2,
  Eye, EyeOff, RefreshCw, Info,
} from 'lucide-react';
import { LoadingSpinner } from '@/components/LoadingSpinner';
import { settingsService, scannerService, type Setting, type FolderRule } from '@/services/settingsService';
import { useSettingsStore } from '@/store/settingsStore';
import { useAiStatus } from '@/hooks/useAIAnalysis';
import { toast } from '@/store/toastStore';

export function Settings() {
  const qc = useQueryClient();
  const { data: settings, isLoading } = useQuery({
    queryKey: ['settings', 'all'],
    queryFn: () => settingsService.all(),
  });
  const { data: aiStatus } = useAiStatus();
  const { data: watched } = useQuery({
    queryKey: ['watched-folders'],
    queryFn: () => scannerService.watchedFolders(),
  });

  const setSetting = useSettingsStore((s) => s.set);
  const [localSettings, setLocalSettings] = useState<Record<string, unknown>>({});

  useEffect(() => {
    if (settings) {
      const map: Record<string, unknown> = {};
      settings.forEach((s: Setting) => {
        map[s.key] = s.value;
      });
      setLocalSettings(map);
      useSettingsStore.getState().setAll(map);
    }
  }, [settings]);

  const saveMutation = useMutation({
    mutationFn: async ({ key, value }: { key: string; value: unknown }) => {
      await settingsService.set(key, value);
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['settings'] });
      toast.success('Instelling opgeslagen');
    },
    onError: (e) => toast.error(`Opslaan mislukt: ${e}`),
  });

  if (isLoading) {
    return <LoadingSpinner size={36} />;
  }

  function getStr(key: string, fallback = ''): string {
    const v = localSettings[key];
    if (typeof v === 'string') return v.replace(/^"|"$/g, '');
    return fallback;
  }

  function getBool(key: string, fallback = false): boolean {
    const v = localSettings[key];
    if (typeof v === 'boolean') return v;
    if (typeof v === 'string') return v === 'true';
    return fallback;
  }

  function getNum(key: string, fallback = 0): number {
    const v = localSettings[key];
    if (typeof v === 'number') return v;
    if (typeof v === 'string') return parseFloat(v) || fallback;
    return fallback;
  }

  function updateValue(key: string, value: unknown) {
    setLocalSettings((s) => ({ ...s, [key]: value }));
    setSetting(key, value);
    saveMutation.mutate({ key, value });
  }

  async function addWatchFolder() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Kies een map om te monitoren',
    });
    if (typeof selected !== 'string') return;
    try {
      await scannerService.addWatchFolder(selected);
      qc.invalidateQueries({ queryKey: ['watched-folders'] });
      toast.success('Map toegevoegd');
    } catch (e) {
      toast.error(`Toevoegen mislukt: ${e}`);
    }
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-surface-900">Instellingen</h1>
        <p className="text-surface-500 mt-1">App-configuratie, AI-modellen en mappen</p>
      </div>

      {/* AI status */}
      {aiStatus && (
        <div className="card p-5">
          <h2 className="text-lg font-semibold mb-3 flex items-center gap-2">
            <Cpu className="w-5 h-5" />
            AI-pipeline status
          </h2>
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
            <StatusItem
              label="Objectdetectie (YOLOv8)"
              loaded={aiStatus.objectDetectionLoaded}
            />
            <StatusItem label="CLIP embeddings" loaded={aiStatus.clipLoaded} />
            <StatusItem label="OCR (Tesseract)" loaded={aiStatus.ocrAvailable} />
          </div>
          <div className="mt-4 text-sm">
            <p><strong>Device:</strong> {aiStatus.device}</p>
            <p className="mt-1"><strong>Modelmap:</strong> <code className="text-xs bg-surface-50 p-1 rounded">{aiStatus.modelDir}</code></p>
            <p className="mt-2 text-surface-500 text-xs flex items-start gap-2">
              <Info className="w-4 h-4 mt-0.5 flex-shrink-0" />
              <span>
                Plaats ONNX-modelbestanden in de modelmap om AI-functies in te schakelen.
                Zie <code>INSTALL.md</code> voor instructies. Zonder modellen draait de app
                nog steeds, maar beperkt tot heuristische classificatie en OCR.
              </span>
            </p>
          </div>
        </div>
      )}

      {/* Watch folders */}
      <div className="card p-5">
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-lg font-semibold flex items-center gap-2">
            <FolderOpen className="w-5 h-5" />
            Gemonitorde mappen
          </h2>
          <button onClick={addWatchFolder} className="btn-primary text-sm">
            <FolderPlus className="w-4 h-4" />
            Map toevoegen
          </button>
        </div>
        {watched && watched.length > 0 ? (
          <div className="space-y-2">
            {watched.map((folder) => (
              <div key={folder} className="flex items-center gap-3 p-2 rounded border border-surface-200">
                <FolderOpen className="w-4 h-4 text-surface-400 flex-shrink-0" />
                <span className="text-sm font-mono flex-1 truncate">{folder}</span>
                <button
                  onClick={async () => {
                    await scannerService.removeWatchFolder(folder);
                    qc.invalidateQueries({ queryKey: ['watched-folders'] });
                    toast.success('Map verwijderd');
                  }}
                  className="btn-ghost text-red-600"
                >
                  <Trash2 className="w-4 h-4" />
                </button>
              </div>
            ))}
          </div>
        ) : (
          <p className="text-sm text-surface-500">
            Nog geen mappen toegevoegd. Voeg mappen toe om snel te scannen.
          </p>
        )}
      </div>

      {/* General settings */}
      <div className="card p-5">
        <h2 className="text-lg font-semibold mb-3 flex items-center gap-2">
          <Database className="w-5 h-5" />
          Algemene instellingen
        </h2>
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div>
            <label className="label">Taal</label>
            <select
              value={getStr('app.language', 'nl-NL')}
              onChange={(e) => updateValue('app.language', e.target.value)}
              className="input"
            >
              <option value="nl-NL">Nederlands</option>
              <option value="en-US">English (US)</option>
            </select>
          </div>
          <div>
            <label className="label">Thema</label>
            <select
              value={getStr('app.theme', 'light')}
              onChange={(e) => updateValue('app.theme', e.target.value)}
              className="input"
            >
              <option value="light">Licht</option>
              <option value="dark">Donker</option>
              <option value="system">Systeem</option>
            </select>
          </div>
          <div>
            <label className="label">Thumbnail-grootte (px)</label>
            <input
              type="number"
              value={getNum('app.thumbnail_size', 400)}
              onChange={(e) => updateValue('app.thumbnail_size', parseInt(e.target.value) || 400)}
              className="input"
              min={100}
              max={1000}
            />
          </div>
          <div>
            <label className="label">Thumbnail-kwaliteit (1-100)</label>
            <input
              type="number"
              value={getNum('app.thumbnail_quality', 85)}
              onChange={(e) => updateValue('app.thumbnail_quality', parseInt(e.target.value) || 85)}
              className="input"
              min={1}
              max={100}
            />
          </div>
        </div>
      </div>

      {/* AI settings */}
      <div className="card p-5">
        <h2 className="text-lg font-semibold mb-3 flex items-center gap-2">
          <Cpu className="w-5 h-5" />
          AI-instellingen
        </h2>
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div>
            <label className="label">Objectdetectie-model</label>
            <input
              type="text"
              value={getStr('ai.model_object_detection', 'yolov8n.onnx')}
              onChange={(e) => updateValue('ai.model_object_detection', e.target.value)}
              className="input font-mono text-sm"
            />
          </div>
          <div>
            <label className="label">CLIP-model</label>
            <input
              type="text"
              value={getStr('ai.model_classification', 'clip-vit-base.onnx')}
              onChange={(e) => updateValue('ai.model_classification', e.target.value)}
              className="input font-mono text-sm"
            />
          </div>
          <div>
            <label className="label">Confidence-drempel (0.0 - 1.0)</label>
            <input
              type="number"
              step={0.05}
              min={0}
              max={1}
              value={getNum('ai.confidence_threshold', 0.55)}
              onChange={(e) => updateValue('ai.confidence_threshold', parseFloat(e.target.value) || 0.55)}
              className="input"
            />
            <p className="text-xs text-surface-500 mt-1">
              Minimum confidence om media automatisch te classificeren.
            </p>
          </div>
          <div>
            <label className="label">Batch-grootte</label>
            <input
              type="number"
              min={1}
              max={64}
              value={getNum('ai.batch_size', 8)}
              onChange={(e) => updateValue('ai.batch_size', parseInt(e.target.value) || 8)}
              className="input"
            />
          </div>
          <div className="flex items-center gap-2 sm:col-span-2">
            <input
              type="checkbox"
              id="use_gpu"
              checked={getBool('ai.use_gpu', true)}
              onChange={(e) => updateValue('ai.use_gpu', e.target.checked)}
              className="w-4 h-4 rounded"
            />
            <label htmlFor="use_gpu" className="text-sm">GPU-versnelling gebruiken indien beschikbaar</label>
          </div>
          <div className="flex items-center gap-2 sm:col-span-2">
            <input
              type="checkbox"
              id="enable_ollama"
              checked={getBool('ai.enable_ollama', false)}
              onChange={(e) => updateValue('ai.enable_ollama', e.target.checked)}
              className="w-4 h-4 rounded"
            />
            <label htmlFor="enable_ollama" className="text-sm">
              Ollama LLM inschakelen voor uitgebreidere SEO-tekst (optioneel, lokaal)
            </label>
          </div>
          <div>
            <label className="label">Ollama URL</label>
            <input
              type="text"
              value={getStr('ai.ollama_url', 'http://localhost:11434')}
              onChange={(e) => updateValue('ai.ollama_url', e.target.value)}
              className="input font-mono text-sm"
              disabled={!getBool('ai.enable_ollama', false)}
            />
          </div>
          <div>
            <label className="label">Ollama-model</label>
            <input
              type="text"
              value={getStr('ai.ollama_model', 'llama3.1:8b')}
              onChange={(e) => updateValue('ai.ollama_model', e.target.value)}
              className="input font-mono text-sm"
              disabled={!getBool('ai.enable_ollama', false)}
            />
          </div>
        </div>
      </div>

      {/* SEO settings */}
      <div className="card p-5">
        <h2 className="text-lg font-semibold mb-3">SEO-instellingen</h2>
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div>
            <label className="label">Bedrijfsnaam</label>
            <input
              type="text"
              value={getStr('seo.brand_name', 'Mariën Sanitair en Centrale Verwarming')}
              onChange={(e) => updateValue('seo.brand_name', e.target.value)}
              className="input"
            />
          </div>
          <div>
            <label className="label">Website URL</label>
            <input
              type="text"
              value={getStr('seo.website_url', 'https://www.mariensanitair.nl')}
              onChange={(e) => updateValue('seo.website_url', e.target.value)}
              className="input"
            />
          </div>
          <div>
            <label className="label">Contact-email</label>
            <input
              type="email"
              value={getStr('seo.contact_email')}
              onChange={(e) => updateValue('seo.contact_email', e.target.value)}
              className="input"
            />
          </div>
          <div>
            <label className="label">Contact-telefoon</label>
            <input
              type="text"
              value={getStr('seo.contact_phone')}
              onChange={(e) => updateValue('seo.contact_phone', e.target.value)}
              className="input"
            />
          </div>
        </div>
      </div>
    </div>
  );
}

function StatusItem({ label, loaded }: { label: string; loaded: boolean }) {
  return (
    <div className="flex items-center gap-2 p-3 border border-surface-200 rounded-lg">
      {loaded ? (
        <div className="w-2.5 h-2.5 rounded-full bg-green-500" />
      ) : (
        <div className="w-2.5 h-2.5 rounded-full bg-red-500" />
      )}
      <span className="text-sm font-medium">{label}</span>
      <span className="text-xs text-surface-500 ml-auto">
        {loaded ? 'Actief' : 'Niet geladen'}
      </span>
    </div>
  );
}
