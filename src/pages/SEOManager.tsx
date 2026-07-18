/// SEO Manager page.

import { useState } from 'react';
import { useSeoList, useDeleteSeo } from '@/hooks/useSeo';
import { useProjects } from '@/hooks/useProjects';
import { useGenerateSeo } from '@/hooks/useSeo';
import { LoadingSpinner } from '@/components/LoadingSpinner';
import { EmptyState } from '@/components/EmptyState';
import { Modal } from '@/components/Modal';
import {
  FileText, Sparkles, Trash2, Search, Copy,
  CheckCircle2, Clock, Eye, Download,
} from 'lucide-react';
import type { Seo, SeoStatus } from '@/types';
import { SEO_STATUS_LABELS, SEO_STATUS_COLORS } from '@/types';
import { seoService } from '@/services/seoService';
import { toast } from '@/store/toastStore';
import { formatDate } from '@/lib/format';

export function SEOManager() {
  const [status, setStatus] = useState<SeoStatus | undefined>();
  const [search, setSearch] = useState('');
  const [preview, setPreview] = useState<Seo | null>(null);
  const { data: seoList, isLoading } = useSeoList(status);
  const { data: projects } = useProjects({ limit: 200 });
  const del = useDeleteSeo();
  const generate = useGenerateSeo();
  const [genProjectId, setGenProjectId] = useState<string>('');

  async function handleGenerate() {
    if (!genProjectId) {
      toast.warning('Selecteer eerst een project');
      return;
    }
    try {
      const result = await generate.mutateAsync(genProjectId);
      setPreview(result.seo);
      toast.success('SEO-content gegenereerd');
    } catch (e) {
      toast.error(`Genereren mislukt: ${e}`);
    }
  }

  async function handleExport(seo: Seo) {
    try {
      const md = await seoService.exportMarkdown(seo.id);
      // Use Tauri save dialog (omitted for brevity — in production use @tauri-apps/plugin-dialog save())
      const blob = new Blob([md], { type: 'text/markdown' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${seo.slug}.md`;
      a.click();
      URL.revokeObjectURL(url);
      toast.success('Markdown geëxporteerd');
    } catch (e) {
      toast.error(`Export mislukt: ${e}`);
    }
  }

  if (isLoading) {
    return <LoadingSpinner size={36} />;
  }

  const filtered = (seoList ?? []).filter((s) =>
    !search || s.title.toLowerCase().includes(search.toLowerCase()) || s.slug.includes(search.toLowerCase()),
  );

  return (
    <div className="space-y-6">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold text-surface-900">SEO-manager</h1>
          <p className="text-surface-500 mt-1">
            Nederlandse SEO-content voor projecten — concept, klaar of gepubliceerd
          </p>
        </div>
      </div>

      {/* Generate new */}
      <div className="card p-5">
        <h2 className="text-lg font-semibold mb-3 flex items-center gap-2">
          <Sparkles className="w-5 h-5 text-brand-600" />
          Nieuwe SEO-content genereren
        </h2>
        <div className="flex gap-2">
          <select
            value={genProjectId}
            onChange={(e) => setGenProjectId(e.target.value)}
            className="input flex-1"
          >
            <option value="">Kies een project...</option>
            {projects?.map((p) => (
              <option key={p.id} value={p.id}>{p.name}</option>
            ))}
          </select>
          <button onClick={handleGenerate} disabled={generate.isPending} className="btn-primary">
            {generate.isPending ? <Clock className="w-4 h-4 animate-spin" /> : <Sparkles className="w-4 h-4" />}
            Genereer
          </button>
        </div>
      </div>

      {/* Filters */}
      <div className="card p-4 flex flex-wrap items-center gap-3">
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-surface-400" />
          <input
            type="search"
            placeholder="Zoek SEO-content..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="input pl-10"
          />
        </div>
        <select
          value={status ?? ''}
          onChange={(e) => setStatus((e.target.value || undefined) as SeoStatus | undefined)}
          className="input max-w-xs"
        >
          <option value="">Alle statussen</option>
          {Object.entries(SEO_STATUS_LABELS).map(([k, v]) => (
            <option key={k} value={k}>{v}</option>
          ))}
        </select>
      </div>

      {/* List */}
      {filtered.length === 0 ? (
        <EmptyState
          icon={FileText}
          title="Geen SEO-content"
          description="Selecteer een project hierboven om Nederlandse SEO-content te genereren."
        />
      ) : (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
          {filtered.map((seo) => (
            <div key={seo.id} className="card p-5">
              <div className="flex items-start justify-between gap-3 mb-2">
                <h3 className="font-semibold text-surface-900 line-clamp-1">{seo.title}</h3>
                <span className={`badge ${SEO_STATUS_COLORS[seo.status]}`}>
                  {SEO_STATUS_LABELS[seo.status]}
                </span>
              </div>
              <p className="text-sm text-surface-600 line-clamp-2 mb-3">
                {seo.metaDescription}
              </p>
              <div className="flex flex-wrap gap-1 mb-3">
                {seo.keywords.slice(0, 5).map((k) => (
                  <span key={k} className="badge bg-surface-100 text-surface-700 text-xs">
                    {k}
                  </span>
                ))}
                {seo.keywords.length > 5 && (
                  <span className="badge bg-surface-100 text-surface-500 text-xs">
                    +{seo.keywords.length - 5}
                  </span>
                )}
              </div>
              <div className="flex items-center justify-between text-xs text-surface-500 mb-3">
                <span>/{seo.slug}</span>
                <span>{formatDate(seo.updatedAt)}</span>
              </div>
              <div className="flex gap-2">
                <button onClick={() => setPreview(seo)} className="btn-secondary text-xs flex-1">
                  <Eye className="w-3 h-3" />
                  Bekijk
                </button>
                <button onClick={() => handleExport(seo)} className="btn-secondary text-xs">
                  <Download className="w-3 h-3" />
                </button>
                <button
                  onClick={() => {
                    if (confirm('SEO verwijderen?')) del.mutate(seo.id);
                  }}
                  className="btn-danger text-xs"
                >
                  <Trash2 className="w-3 h-3" />
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Preview modal */}
      <Modal
        open={!!preview}
        onClose={() => setPreview(null)}
        title={preview?.title}
        size="lg"
      >
        {preview && <SeoPreview seo={preview} />}
      </Modal>
    </div>
  );
}

function SeoPreview({ seo }: { seo: Seo }) {
  const [tab, setTab] = useState<'preview' | 'markdown' | 'html' | 'json'>('preview');

  return (
    <div className="space-y-4">
      <div className="flex gap-1 border-b border-surface-200">
        {(['preview', 'markdown', 'html', 'json'] as const).map((t) => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className={`px-3 py-2 text-sm font-medium border-b-2 transition-colors ${
              tab === t
                ? 'border-brand-600 text-brand-600'
                : 'border-transparent text-surface-500 hover:text-surface-700'
            }`}
          >
            {t === 'preview' ? 'Voorbeeld' : t.toUpperCase()}
          </button>
        ))}
      </div>

      {tab === 'preview' && (
        <div className="space-y-4">
          <div className="border border-surface-200 rounded-lg p-4">
            <p className="text-xs text-surface-500 mb-1">SERP voorbeeld</p>
            <h3 className="text-lg text-brand-700 hover:underline cursor-pointer">{seo.title}</h3>
            <p className="text-xs text-green-700 mt-0.5">{seo.canonicalUrl ?? `/${seo.slug}`}</p>
            <p className="text-sm text-surface-600 mt-1">{seo.metaDescription}</p>
          </div>

          <div>
            <p className="text-sm font-medium text-surface-700 mb-2">Keywords</p>
            <div className="flex flex-wrap gap-1">
              {seo.keywords.map((k) => (
                <span key={k} className="badge bg-brand-50 text-brand-700">{k}</span>
              ))}
            </div>
          </div>

          {seo.bodyHtml && (
            <div>
              <p className="text-sm font-medium text-surface-700 mb-2">Body</p>
              <div
                className="prose prose-sm max-w-none"
                dangerouslySetInnerHTML={{ __html: seo.bodyHtml }}
              />
            </div>
          )}
        </div>
      )}

      {tab === 'markdown' && (
        <pre className="bg-surface-50 p-4 rounded-lg text-xs overflow-auto max-h-[60vh] whitespace-pre-wrap">
          {seo.bodyMarkdown}
        </pre>
      )}

      {tab === 'html' && (
        <pre className="bg-surface-50 p-4 rounded-lg text-xs overflow-auto max-h-[60vh] whitespace-pre-wrap">
          {seo.bodyHtml}
        </pre>
      )}

      {tab === 'json' && (
        <pre className="bg-surface-50 p-4 rounded-lg text-xs overflow-auto max-h-[60vh]">
          {JSON.stringify({
            id: seo.id,
            projectId: seo.projectId,
            title: seo.title,
            slug: seo.slug,
            metaDescription: seo.metaDescription,
            keywords: seo.keywords,
            canonicalUrl: seo.canonicalUrl,
            ogTitle: seo.ogTitle,
            ogDescription: seo.ogDescription,
            language: seo.language,
            readingTimeMin: seo.readingTimeMin,
            wordCount: seo.wordCount,
            status: seo.status,
            schemaOrg: seo.schemaOrgJson ? JSON.parse(seo.schemaOrgJson) : null,
            altTexts: seo.altTexts,
          }, null, 2)}
        </pre>
      )}
    </div>
  );
}
