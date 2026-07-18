/// Dashboard page — overview of all media, projects, AI status.

import { useQuery } from '@tanstack/react-query';
import { Link } from 'react-router-dom';
import {
  Images, Briefcase, Lock, Copy, FolderKanban,
  Cpu, ScanLine, BrainCircuit, FileText, AlertCircle, CheckCircle2,
} from 'lucide-react';
import { StatCard } from '@/components/StatCard';
import { LoadingSpinner } from '@/components/LoadingSpinner';
import { EmptyState } from '@/components/EmptyState';
import { ProjectCard } from '@/components/ProjectCard';
import { statsService } from '@/services/settingsService';
import { projectService } from '@/services/projectService';
import { formatNumber, formatPercent } from '@/lib/format';

export function Dashboard() {
  const { data: stats, isLoading } = useQuery({
    queryKey: ['stats', 'dashboard'],
    queryFn: () => statsService.dashboard(),
  });

  const { data: projects } = useQuery({
    queryKey: ['projects', 'recent'],
    queryFn: () => projectService.list({ limit: 6 }),
  });

  if (isLoading || !stats) {
    return <LoadingSpinner size={36} />;
  }

  const aiReady =
    stats.aiStatus.objectDetectionLoaded &&
    stats.aiStatus.clipLoaded &&
    stats.aiStatus.ocrAvailable;

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-2xl font-bold text-surface-900">Dashboard</h1>
        <p className="text-surface-500 mt-1">
          Overzicht van uw mediabibliotheek en AI-analyse
        </p>
      </div>

      {/* AI status banner */}
      <div
        className={`rounded-lg p-4 border ${
          aiReady
            ? 'bg-green-50 border-green-200 text-green-800'
            : 'bg-yellow-50 border-yellow-200 text-yellow-800'
        }`}
      >
        <div className="flex items-center gap-3">
          {aiReady ? (
            <CheckCircle2 className="w-5 h-5" />
          ) : (
            <AlertCircle className="w-5 h-5" />
          )}
          <div className="flex-1">
            <p className="font-medium">
              AI-pipeline: {aiReady ? 'volledig operationeel' : 'gedeeltelijk actief'}
            </p>
            <p className="text-sm mt-0.5">
              Object detectie: {stats.aiStatus.objectDetectionLoaded ? '✓' : '✗'} ·
              CLIP: {stats.aiStatus.clipLoaded ? '✓' : '✗'} ·
              OCR: {stats.aiStatus.ocrAvailable ? '✓' : '✗'} ·
              Device: {stats.aiStatus.device}
            </p>
          </div>
          <Link to="/settings" className="btn-secondary">
            Instellingen
          </Link>
        </div>
        {!aiReady && stats.aiStatus.errors.length > 0 && (
          <ul className="mt-3 text-sm list-disc list-inside space-y-0.5">
            {stats.aiStatus.errors.slice(0, 3).map((e, i) => (
              <li key={i}>{e}</li>
            ))}
          </ul>
        )}
      </div>

      {/* Stats grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          label="Totaal media"
          value={formatNumber(stats.totalMedia)}
          icon={Images}
          color="brand"
        />
        <StatCard
          label="Zakelijk"
          value={formatNumber(stats.businessCount)}
          icon={Briefcase}
          color="green"
          hint={`${formatPercent(stats.totalMedia > 0 ? stats.businessCount / stats.totalMedia : 0)} van totaal`}
        />
        <StatCard
          label="Ongeclassificeerd"
          value={formatNumber(stats.unclassifiedCount)}
          icon={BrainCircuit}
          color="orange"
          hint={stats.unclassifiedCount > 0 ? 'Klaar voor AI-analyse' : 'Alles geclassificeerd'}
        />
        <StatCard
          label="Projecten"
          value={formatNumber(stats.projectCount)}
          icon={FolderKanban}
          color="brand"
          hint={`${stats.detectedProjects} gedetecteerd, ${stats.approvedProjects} goedgekeurd`}
        />
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          label="Prive"
          value={formatNumber(stats.privateCount)}
          icon={Lock}
          color="gray"
        />
        <StatCard
          label="Duplicaten"
          value={formatNumber(stats.duplicatesCount)}
          icon={Copy}
          color="gray"
        />
        <StatCard
          label="Afbeeldingen"
          value={formatNumber(stats.imagesCount)}
          icon={Images}
          color="brand"
        />
        <StatCard
          label="Video's"
          value={formatNumber(stats.videosCount)}
          icon={FileText}
          color="brand"
        />
      </div>

      {/* Quick actions */}
      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <Link to="/media" className="card p-5 hover:shadow-md transition-shadow">
          <ScanLine className="w-8 h-8 text-brand-600 mb-3" />
          <h3 className="font-semibold text-surface-900">Map importeren</h3>
          <p className="text-sm text-surface-500 mt-1">
            Scan een iCloud-map of andere map en voeg media automatisch toe.
          </p>
        </Link>
        <Link to="/ai" className="card p-5 hover:shadow-md transition-shadow">
          <BrainCircuit className="w-8 h-8 text-brand-600 mb-3" />
          <h3 className="font-semibold text-surface-900">AI-analyse draaien</h3>
          <p className="text-sm text-surface-500 mt-1">
            Classificeer zakelijk/prive en detecteer installatieprojecten.
          </p>
        </Link>
        <Link to="/seo" className="card p-5 hover:shadow-md transition-shadow">
          <FileText className="w-8 h-8 text-brand-600 mb-3" />
          <h3 className="font-semibold text-surface-900">SEO-content genereren</h3>
          <p className="text-sm text-surface-500 mt-1">
            Genereer Nederlandse projectbeschrijvingen, meta-tags en alt-teksten.
          </p>
        </Link>
      </div>

      {/* Recent projects */}
      <div>
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-surface-900">Recente projecten</h2>
          <Link to="/projects" className="text-sm text-brand-600 hover:underline">
            Alle projecten →
          </Link>
        </div>
        {projects && projects.length > 0 ? (
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
            {projects.map((p) => (
              <ProjectCard key={p.id} project={{
                id: p.id,
                name: p.name,
                slug: p.slug,
                status: p.status,
                projectType: p.projectType,
                confidence: p.confidence,
                mediaCount: 0,
                firstDate: p.startDate,
                lastDate: p.endDate,
              }} />
            ))}
          </div>
        ) : (
          <EmptyState
            icon={FolderKanban}
            title="Nog geen projecten"
            description="Importeer media en draai AI-detectie om projecten te laten verschijnen."
          />
        )}
      </div>
    </div>
  );
}
