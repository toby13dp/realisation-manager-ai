/// Privacy Center page.

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import {
  ShieldCheck, Eye, EyeOff, Lock, Unlock, FolderLock,
  Plus, Trash2, ShieldAlert, CheckCircle2,
} from 'lucide-react';
import { LoadingSpinner } from '@/components/LoadingSpinner';
import { EmptyState } from '@/components/EmptyState';
import { Modal } from '@/components/Modal';
import { settingsService, type FolderRule } from '@/services/settingsService';
import { mediaService } from '@/services/mediaService';
import { useSettingsStore } from '@/store/settingsStore';
import { toast } from '@/store/toastStore';
import { formatNumber } from '@/lib/format';

export function PrivacyCenter() {
  const qc = useQueryClient();
  const privacyMode = useSettingsStore((s) => s.privacyMode);
  const togglePrivacyMode = useSettingsStore((s) => s.togglePrivacyMode);

  const { data: rules, isLoading } = useQuery({
    queryKey: ['folder-rules'],
    queryFn: () => settingsService.listFolderRules(),
  });

  const { data: privateMedia } = useQuery({
    queryKey: ['media', 'private'],
    queryFn: () => mediaService.list({ isPrivate: true, limit: 5000 }),
  });

  const [showAddRule, setShowAddRule] = useState(false);
  const [newRule, setNewRule] = useState<Partial<FolderRule>>({
    folderPath: '',
    classification: 'private',
    recursive: true,
    priority: 10,
  });

  const togglePrivacyModeMutation = useMutation({
    mutationFn: async (next: boolean) => {
      await settingsService.set('app.privacy_mode', next);
      return next;
    },
    onSuccess: (next) => {
      togglePrivacyMode();
      qc.invalidateQueries({ queryKey: ['media'] });
      toast.success(next ? 'Privacymodus ingeschakeld' : 'Privacymodus uitgeschakeld');
    },
    onError: (e) => toast.error(`Fout: ${e}`),
  });

  const saveRule = useMutation({
    mutationFn: (rule: FolderRule) => settingsService.upsertFolderRule(rule),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['folder-rules'] });
      toast.success('Regel opgeslagen');
      setShowAddRule(false);
      setNewRule({ folderPath: '', classification: 'private', recursive: true, priority: 10 });
    },
    onError: (e) => toast.error(`Opslaan mislukt: ${e}`),
  });

  const deleteRule = useMutation({
    mutationFn: (id: string) => settingsService.deleteFolderRule(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['folder-rules'] });
      toast.success('Regel verwijderd');
    },
    onError: (e) => toast.error(`Verwijderen mislukt: ${e}`),
  });

  if (isLoading) {
    return <LoadingSpinner size={36} />;
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-surface-900">Privacycentrum</h1>
        <p className="text-surface-500 mt-1">
          Beheer privacymodus, mapregels en privé-media
        </p>
      </div>

      {/* Privacy mode toggle */}
      <div className={`card p-6 ${privacyMode ? 'bg-brand-50 border-brand-200' : ''}`}>
        <div className="flex items-center gap-4">
          <div className={`rounded-lg p-3 ${privacyMode ? 'bg-brand-600 text-white' : 'bg-surface-100 text-surface-700'}`}>
            {privacyMode ? <EyeOff className="w-6 h-6" /> : <Eye className="w-6 h-6" />}
          </div>
          <div className="flex-1">
            <h2 className="text-lg font-semibold">Privacymodus</h2>
            <p className="text-sm text-surface-600 mt-1">
              Wanneer ingeschakeld worden alle privé-media verborgen in elke weergave,
              export en AI-suggestie. De media worden niet verwijderd.
            </p>
          </div>
          <button
            onClick={() => togglePrivacyModeMutation.mutate(!privacyMode)}
            disabled={togglePrivacyModeMutation.isPending}
            className={privacyMode ? 'btn-primary' : 'btn-secondary'}
          >
            {privacyMode ? 'Uitschakelen' : 'Inschakelen'}
          </button>
        </div>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <div className="card p-5">
          <Lock className="w-8 h-8 text-purple-600 mb-2" />
          <p className="text-2xl font-bold">{formatNumber(privateMedia?.length ?? 0)}</p>
          <p className="text-sm text-surface-500">Privé-media</p>
        </div>
        <div className="card p-5">
          <FolderLock className="w-8 h-8 text-brand-600 mb-2" />
          <p className="text-2xl font-bold">{rules?.length ?? 0}</p>
          <p className="text-sm text-surface-500">Mapregels</p>
        </div>
        <div className="card p-5">
          <ShieldCheck className="w-8 h-8 text-green-600 mb-2" />
          <p className="text-2xl font-bold">{privacyMode ? 'AAN' : 'UIT'}</p>
          <p className="text-sm text-surface-500">Privacymodus</p>
        </div>
      </div>

      {/* Folder rules */}
      <div>
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-surface-900">Mapregels</h2>
          <button onClick={() => setShowAddRule(true)} className="btn-primary">
            <Plus className="w-4 h-4" />
            Regel toevoegen
          </button>
        </div>

        {rules && rules.length > 0 ? (
          <div className="space-y-2">
            {rules.map((rule) => (
              <div key={rule.id} className="card p-4 flex items-center gap-4">
                <div className={`rounded p-2 ${
                  rule.classification === 'private' ? 'bg-purple-100 text-purple-700' :
                  rule.classification === 'business' ? 'bg-brand-100 text-brand-700' :
                  'bg-red-100 text-red-700'
                }`}>
                  <FolderLock className="w-5 h-5" />
                </div>
                <div className="flex-1 min-w-0">
                  <p className="font-mono text-sm truncate">{rule.folderPath}</p>
                  <p className="text-xs text-surface-500 mt-0.5">
                    {rule.classification === 'private' ? 'Markeer als prive' :
                     rule.classification === 'business' ? 'Markeer als zakelijk' :
                     'Uitsluiten van import'}
                    {rule.recursive ? ' · inclusief submappen' : ''}
                    {rule.priority ? ` · prioriteit ${rule.priority}` : ''}
                  </p>
                </div>
                <button
                  onClick={() => deleteRule.mutate(rule.id)}
                  className="btn-ghost text-red-600"
                >
                  <Trash2 className="w-4 h-4" />
                </button>
              </div>
            ))}
          </div>
        ) : (
          <EmptyState
            icon={FolderLock}
            title="Geen mapregels"
            description="Voeg regels toe om mappen automatisch als privé of zakelijk te markeren. Bestanden in deze mappen worden nooit door AI overschreven."
            action={
              <button onClick={() => setShowAddRule(true)} className="btn-primary">
                <Plus className="w-4 h-4" />
                Regel toevoegen
              </button>
            }
          />
        )}
      </div>

      {/* Add rule modal */}
      <Modal
        open={showAddRule}
        onClose={() => setShowAddRule(false)}
        title="Mapregel toevoegen"
        size="md"
        footer={
          <>
            <button onClick={() => setShowAddRule(false)} className="btn-secondary">Annuleren</button>
            <button
              onClick={() => {
                if (!newRule.folderPath) {
                  toast.warning('Vul een mappad in');
                  return;
                }
                saveRule.mutate({
                  id: crypto.randomUUID(),
                  folderPath: newRule.folderPath!,
                  classification: newRule.classification as FolderRule['classification'],
                  recursive: newRule.recursive ?? true,
                  priority: newRule.priority ?? 0,
                  notes: null,
                  createdAt: new Date().toISOString(),
                  updatedAt: new Date().toISOString(),
                });
              }}
              className="btn-primary"
            >
              Opslaan
            </button>
          </>
        }
      >
        <div className="space-y-4">
          <div>
            <label className="label">Mappad</label>
            <input
              type="text"
              value={newRule.folderPath ?? ''}
              onChange={(e) => setNewRule({ ...newRule, folderPath: e.target.value })}
              placeholder="C:\Users\Naam\Pictures\Prive"
              className="input font-mono text-sm"
            />
            <p className="text-xs text-surface-500 mt-1">
              Volledig pad naar de map die u wilt markeren.
            </p>
          </div>
          <div>
            <label className="label">Classificatie</label>
            <select
              value={newRule.classification}
              onChange={(e) => setNewRule({ ...newRule, classification: e.target.value as FolderRule['classification'] })}
              className="input"
            >
              <option value="private">Markeer als prive</option>
              <option value="business">Markeer als zakelijk</option>
              <option value="exclude">Uitsluiten van import</option>
            </select>
          </div>
          <div className="flex items-center gap-2">
            <input
              type="checkbox"
              id="recursive"
              checked={newRule.recursive ?? true}
              onChange={(e) => setNewRule({ ...newRule, recursive: e.target.checked })}
              className="w-4 h-4 rounded"
            />
            <label htmlFor="recursive" className="text-sm">
              Inclusief submappen
            </label>
          </div>
          <div>
            <label className="label">Prioriteit (0-100)</label>
            <input
              type="number"
              min={0}
              max={100}
              value={newRule.priority ?? 0}
              onChange={(e) => setNewRule({ ...newRule, priority: parseInt(e.target.value) || 0 })}
              className="input"
            />
            <p className="text-xs text-surface-500 mt-1">
              Hogere prioriteit wint bij conflicterende regels.
            </p>
          </div>
        </div>
      </Modal>
    </div>
  );
}
