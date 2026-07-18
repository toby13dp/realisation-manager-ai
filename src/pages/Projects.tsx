/// Projects list page.

import { useProjects } from '@/hooks/useProjects';
import { useDetectProjects } from '@/hooks/useAIAnalysis';
import { ProjectCard } from '@/components/ProjectCard';
import { LoadingSpinner } from '@/components/LoadingSpinner';
import { EmptyState } from '@/components/EmptyState';
import { Sparkles, FolderKanban } from 'lucide-react';
import { useState } from 'react';
import type { ProjectStatus, ProjectType } from '@/types';
import { PROJECT_STATUS_LABELS, PROJECT_TYPE_LABELS } from '@/types';
import { projectService } from '@/services/projectService';
import { useQueryClient } from '@tanstack/react-query';
import { toast } from '@/store/toastStore';

export function Projects() {
  const [status, setStatus] = useState<ProjectStatus | undefined>();
  const [projectType, setProjectType] = useState<ProjectType | undefined>();
  const { data: projects, isLoading } = useProjects({ status, projectType, limit: 200 });
  const detect = useDetectProjects();
  const qc = useQueryClient();

  async function handleDetect() {
    try {
      const detected = await detect.mutateAsync(true);
      toast.success(`${detected.length} projecten gedetecteerd`);
    } catch (e) {
      toast.error(`Detectie mislukt: ${e}`);
    }
  }

  if (isLoading) {
    return <LoadingSpinner size={36} />;
  }

  return (
    <div className="space-y-6">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold text-surface-900">Projecten</h1>
          <p className="text-surface-500 mt-1">
            {projects?.length ?? 0} projecten
          </p>
        </div>
        <button onClick={handleDetect} disabled={detect.isPending} className="btn-primary">
          <Sparkles className="w-4 h-4" />
          {detect.isPending ? 'Detecteren...' : 'Projecten detecteren'}
        </button>
      </div>

      {/* Filters */}
      <div className="card p-4 flex flex-wrap items-center gap-3">
        <span className="text-sm text-surface-600">Status:</span>
        <select
          value={status ?? ''}
          onChange={(e) => setStatus((e.target.value || undefined) as ProjectStatus | undefined)}
          className="input max-w-xs"
        >
          <option value="">Alle statussen</option>
          {Object.entries(PROJECT_STATUS_LABELS).map(([k, v]) => (
            <option key={k} value={k}>{v}</option>
          ))}
        </select>

        <span className="text-sm text-surface-600 ml-4">Type:</span>
        <select
          value={projectType ?? ''}
          onChange={(e) => setProjectType((e.target.value || undefined) as ProjectType | undefined)}
          className="input max-w-xs"
        >
          <option value="">Alle typen</option>
          {Object.entries(PROJECT_TYPE_LABELS).map(([k, v]) => (
            <option key={k} value={k}>{v}</option>
          ))}
        </select>
      </div>

      {projects && projects.length > 0 ? (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
          {projects.map((p) => (
            <ProjectCard
              key={p.id}
              project={{
                id: p.id,
                name: p.name,
                slug: p.slug,
                status: p.status,
                projectType: p.projectType,
                confidence: p.confidence,
                mediaCount: 0,
                firstDate: p.startDate,
                lastDate: p.endDate,
              }}
            />
          ))}
        </div>
      ) : (
        <EmptyState
          icon={FolderKanban}
          title="Geen projecten"
          description="Draai AI-detectie om automatisch projecten te groeperen uit uw zakelijke media."
          action={
            <button onClick={handleDetect} disabled={detect.isPending} className="btn-primary">
              <Sparkles className="w-4 h-4" />
              {detect.isPending ? 'Bezig...' : 'Detecteer projecten'}
            </button>
          }
        />
      )}
    </div>
  );
}
