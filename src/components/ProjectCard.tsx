/// Project card for project grid.

import { Link } from 'react-router-dom';
import { Calendar, MapPin, ImageIcon, FolderKanban } from 'lucide-react';
import type { ProjectSummary } from '@/types';
import { PROJECT_TYPE_LABELS, PROJECT_TYPE_COLORS, PROJECT_STATUS_LABELS } from '@/types';
import { ConfidenceBadge } from './ConfidenceBadge';
import { formatDate } from '@/lib/format';
import { cn } from '@/lib/utils';

interface ProjectCardProps {
  project: ProjectSummary;
}

export function ProjectCard({ project }: ProjectCardProps) {
  return (
    <Link
      to={`/projects/${project.id}`}
      className="card p-5 hover:shadow-md transition-shadow block"
    >
      <div className="flex items-start justify-between gap-3 mb-3">
        <div className="flex-1 min-w-0">
          <h3 className="font-semibold text-surface-900 truncate">{project.name}</h3>
          <p className="text-xs text-surface-500 mt-0.5">{project.slug}</p>
        </div>
        <ConfidenceBadge value={project.confidence} />
      </div>

      <div className="flex flex-wrap items-center gap-2 mb-3">
        <span className={cn('badge', PROJECT_TYPE_COLORS[project.projectType])}>
          {PROJECT_TYPE_LABELS[project.projectType]}
        </span>
        <span className="badge bg-surface-100 text-surface-700">
          {PROJECT_STATUS_LABELS[project.status]}
        </span>
      </div>

      <div className="grid grid-cols-2 gap-2 text-xs text-surface-600">
        <div className="flex items-center gap-1.5">
          <ImageIcon className="w-3.5 h-3.5" />
          <span>{project.mediaCount} media</span>
        </div>
        <div className="flex items-center gap-1.5">
          <Calendar className="w-3.5 h-3.5" />
          <span>{formatDate(project.firstDate)}</span>
        </div>
      </div>
    </Link>
  );
}

export function ProjectCardEmpty() {
  return (
    <div className="card p-5 text-center text-surface-500">
      <FolderKanban className="w-8 h-8 mx-auto mb-2 text-surface-300" />
      <p className="text-sm">Geen projecten</p>
    </div>
  );
}
