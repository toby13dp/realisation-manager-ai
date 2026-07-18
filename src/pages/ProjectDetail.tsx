/// Project detail page.

import { useParams, useNavigate, Link } from 'react-router-dom';
import { useProject, useUpdateProject, useApproveProject, useDeleteProject, useAssignMedia } from '@/hooks/useProjects';
import { useMediaList } from '@/hooks/useMedia';
import { useGenerateSeo } from '@/hooks/useSeo';
import { LoadingSpinner } from '@/components/LoadingSpinner';
import { EmptyState } from '@/components/EmptyState';
import { MediaGrid } from '@/components/MediaGrid';
import { ConfidenceBadge } from '@/components/ConfidenceBadge';
import { Modal } from '@/components/Modal';
import {
  ArrowLeft, Calendar, MapPin, User, Mail, Phone, Save, Trash2,
  CheckCircle2, FileText, Sparkles, Tag,
} from 'lucide-react';
import { useState } from 'react';
import { PROJECT_TYPE_LABELS, PROJECT_STATUS_LABELS } from '@/types';
import { formatDate } from '@/lib/format';
import { toast } from '@/store/toastStore';

export function ProjectDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { data: project, isLoading } = useProject(id ?? null);
  const { data: media } = useMediaList({ projectId: id, limit: 5000 });
  const update = useUpdateProject();
  const approve = useApproveProject();
  const del = useDeleteProject();
  const generateSeo = useGenerateSeo();

  const [editMode, setEditMode] = useState(false);
  const [draft, setDraft] = useState(project);
  const [seoModal, setSeoModal] = useState(false);

  if (isLoading) return <LoadingSpinner size={36} />;
  if (!project) {
    return (
      <EmptyState
        title="Project niet gevonden"
        description="Het project is mogelijk verwijderd."
        action={<Link to="/projects" className="btn-primary">Terug naar projecten</Link>}
      />
    );
  }

  if (!draft) setDraft(project);

  async function handleSave() {
    if (!draft) return;
    try {
      await update.mutateAsync(draft);
      setEditMode(false);
    } catch (e) {
      toast.error(`Opslaan mislukt: ${e}`);
    }
  }

  async function handleApprove() {
    if (!id) return;
    await approve.mutateAsync(id);
  }

  async function handleDelete() {
    if (!id) return;
    if (!confirm('Project verwijderen? Media blijft bewaard maar wordt losgekoppeld.')) return;
    await del.mutateAsync(id);
    navigate('/projects');
  }

  async function handleGenerateSeo() {
    if (!id) return;
    try {
      const result = await generateSeo.mutateAsync(id);
      toast.success('SEO-content gegenereerd');
      setSeoModal(true);
    } catch (e) {
      toast.error(`SEO-generatie mislukt: ${e}`);
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between gap-4">
        <button onClick={() => navigate('/projects')} className="btn-ghost">
          <ArrowLeft className="w-4 h-4" />
          Terug
        </button>
        <div className="flex gap-2">
          <button onClick={handleGenerateSeo} disabled={generateSeo.isPending} className="btn-secondary">
            <FileText className="w-4 h-4" />
            {generateSeo.isPending ? 'Bezig...' : 'Genereer SEO'}
          </button>
          {project.status === 'detected' && (
            <button onClick={handleApprove} className="btn-secondary">
              <CheckCircle2 className="w-4 h-4" />
              Goedkeuren
            </button>
          )}
          {editMode ? (
            <button onClick={handleSave} className="btn-primary">
              <Save className="w-4 h-4" />
              Opslaan
            </button>
          ) : (
            <button onClick={() => { setDraft(project); setEditMode(true); }} className="btn-secondary">
              Bewerken
            </button>
          )}
          <button onClick={handleDelete} className="btn-danger">
            <Trash2 className="w-4 h-4" />
          </button>
        </div>
      </div>

      <div className="card p-6">
        <div className="flex items-start justify-between gap-4 mb-4">
          <div className="flex-1">
            {editMode && draft ? (
              <input
                value={draft.name}
                onChange={(e) => setDraft({ ...draft, name: e.target.value })}
                className="input text-xl font-bold"
              />
            ) : (
              <h1 className="text-2xl font-bold text-surface-900">{project.name}</h1>
            )}
            <p className="text-surface-500 mt-1">/{project.slug}</p>
          </div>
          <ConfidenceBadge value={project.confidence} />
        </div>

        <div className="flex flex-wrap items-center gap-2 mb-6">
          <span className="badge bg-surface-100 text-surface-700">
            {PROJECT_TYPE_LABELS[project.projectType]}
          </span>
          <span className="badge bg-surface-100 text-surface-700">
            {PROJECT_STATUS_LABELS[project.status]}
          </span>
          {project.tags.map((t) => (
            <span key={t} className="badge bg-brand-50 text-brand-700">
              <Tag className="w-3 h-3 mr-1" />
              {t}
            </span>
          ))}
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 text-sm">
          <div>
            <p className="text-surface-500 flex items-center gap-1"><Calendar className="w-3.5 h-3.5" /> Startdatum</p>
            <p className="font-medium mt-1">{formatDate(project.startDate)}</p>
          </div>
          <div>
            <p className="text-surface-500 flex items-center gap-1"><Calendar className="w-3.5 h-3.5" /> Einddatum</p>
            <p className="font-medium mt-1">{formatDate(project.endDate)}</p>
          </div>
          <div>
            <p className="text-surface-500 flex items-center gap-1"><MapPin className="w-3.5 h-3.5" /> Locatie</p>
            <p className="font-medium mt-1">{project.locationLabel ?? '—'}</p>
          </div>
          <div>
            <p className="text-surface-500 flex items-center gap-1"><User className="w-3.5 h-3.5" /> Klant</p>
            <p className="font-medium mt-1">{project.customerName ?? '—'}</p>
          </div>
          {project.customerEmail && (
            <div>
              <p className="text-surface-500 flex items-center gap-1"><Mail className="w-3.5 h-3.5" /> Email</p>
              <p className="font-medium mt-1">{project.customerEmail}</p>
            </div>
          )}
          {project.customerPhone && (
            <div>
              <p className="text-surface-500 flex items-center gap-1"><Phone className="w-3.5 h-3.5" /> Telefoon</p>
              <p className="font-medium mt-1">{project.customerPhone}</p>
            </div>
          )}
        </div>
      </div>

      <div>
        <h2 className="text-lg font-semibold text-surface-900 mb-3">
          Media ({media?.length ?? 0})
        </h2>
        {media && media.length > 0 ? (
          <MediaGrid media={media} />
        ) : (
          <EmptyState
            title="Geen media in dit project"
            description="Koppel media via de mediabibliotheek."
          />
        )}
      </div>

      {/* SEO Preview modal */}
      <Modal
        open={seoModal}
        onClose={() => setSeoModal(false)}
        title="SEO-content gegenereerd"
        size="lg"
        footer={
          <>
            <button onClick={() => setSeoModal(false)} className="btn-secondary">Sluiten</button>
            <Link to="/seo" className="btn-primary">Naar SEO-manager</Link>
          </>
        }
      >
        <div className="text-center py-8">
          <Sparkles className="w-12 h-12 text-brand-600 mx-auto mb-3" />
          <p className="text-surface-700">
            SEO-content is gegenereerd en opgeslagen als concept.
            U kunt het bewerken en publiceren via de SEO-manager.
          </p>
        </div>
      </Modal>
    </div>
  );
}
