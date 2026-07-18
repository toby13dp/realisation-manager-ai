/// React Query hooks for projects.

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { projectService } from '@/services/projectService';
import type { Project, ProjectStatus, ProjectType } from '@/types';
import { toast } from '@/store/toastStore';

export function useProjects(params: {
  status?: ProjectStatus;
  projectType?: ProjectType;
  isPrivate?: boolean;
  limit?: number;
} = {}) {
  return useQuery({
    queryKey: ['projects', 'list', params],
    queryFn: () => projectService.list(params),
  });
}

export function useProject(id: string | null) {
  return useQuery({
    queryKey: ['projects', 'get', id],
    queryFn: () => (id ? projectService.get(id) : Promise.resolve(null)),
    enabled: !!id,
  });
}

export function useProjectSummary() {
  return useQuery({
    queryKey: ['projects', 'summary'],
    queryFn: () => projectService.summary(),
  });
}

export function useCreateProject() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (project: Project) => projectService.create(project),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['projects'] });
      toast.success('Project aangemaakt');
    },
    onError: (e) => toast.error(`Aanmaken mislukt: ${e}`),
  });
}

export function useUpdateProject() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (project: Project) => projectService.update(project),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['projects'] });
      toast.success('Project opgeslagen');
    },
    onError: (e) => toast.error(`Opslaan mislukt: ${e}`),
  });
}

export function useDeleteProject() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => projectService.delete(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['projects'] });
      toast.success('Project verwijderd');
    },
    onError: (e) => toast.error(`Verwijderen mislukt: ${e}`),
  });
}

export function useApproveProject() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => projectService.approve(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['projects'] });
      toast.success('Project goedgekeurd');
    },
    onError: (e) => toast.error(`Goedkeuren mislukt: ${e}`),
  });
}

export function useMergeProjects() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ sourceId, targetId }: { sourceId: string; targetId: string }) =>
      projectService.merge(sourceId, targetId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['projects'] });
      toast.success('Projecten samengevoegd');
    },
    onError: (e) => toast.error(`Samenvoegen mislukt: ${e}`),
  });
}

export function useAssignMedia() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ mediaIds, projectId }: { mediaIds: string[]; projectId: string }) =>
      projectService.assignMedia(mediaIds, projectId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['projects'] });
      qc.invalidateQueries({ queryKey: ['media'] });
    },
    onError: (e) => toast.error(`Toewijzen mislukt: ${e}`),
  });
}
